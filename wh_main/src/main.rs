use serenity::client::EventHandler;

#[macro_use]
extern crate wh_core;
extern crate wh_database;
extern crate wh_music;
extern crate wh_points;

extern crate serenity;
#[macro_use]
extern crate log;
extern crate dotenv;
extern crate fern;
extern crate tokio;

struct WhEventHandler;

#[serenity::async_trait]
impl EventHandler for WhEventHandler {
    async fn ready(
        &self,
        _ctx: serenity::client::Context,
        _data_about_bot: serenity::model::gateway::Ready,
    ) {
        info!(
            "Started \x1b[1m{}\x1b[0m on {} guild{}",
            _data_about_bot.user.name,
            _data_about_bot.guilds.len(),
            if _data_about_bot.guilds.len() == 1 {
                ""
            } else {
                "s"
            }
        );
        for guild in _data_about_bot.guilds {
            guild
                .id()
                .disconnect_member(&_ctx.http, &_ctx.cache.current_user_id().await)
                .await
                .expect("Error when disconnecting voice");
        }
    }
}

macro_rules! modules {
    ($list:ident, $($module:ident),*) => {
        let $list = vec![$(&$module::module::MODULE_DECLARATION),*];

    };
}

fn logger_setup() -> Result<(), Box<dyn std::error::Error>> {
    let colors = fern::colors::ColoredLevelConfig::new()
        .error(fern::colors::Color::Red)
        .warn(fern::colors::Color::Yellow)
        .info(fern::colors::Color::Blue)
        .debug(fern::colors::Color::Magenta)
        .trace(fern::colors::Color::BrightWhite);

    Ok(fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[\x1b[1;37m%H:%M:%S\x1b[0m]"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("serenity", log::LevelFilter::Warn)
        .level_for("tracing", log::LevelFilter::Warn)
        .level_for("rustls", log::LevelFilter::Warn)
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("h2", log::LevelFilter::Warn)
        .level_for("reqwest", log::LevelFilter::Warn)
        .level_for("tungstenite", log::LevelFilter::Warn)
        .level_for("sqlx", log::LevelFilter::Warn)
        .level_for("songbird", log::LevelFilter::Warn)
        .level_for("ureq", log::LevelFilter::Warn)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        // Apply globally
        .apply()?)
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    logger_setup().expect("Error when setting up logger");

    bot_launch().await.expect("Error when launching bot");
}

async fn bot_launch() -> Result<(), Box<dyn std::error::Error>> {
    #[serenity::framework::standard::macros::hook]
    async fn after_hook(
        ctx: &serenity::client::Context,
        message: &serenity::model::channel::Message,
        cmd_name: &str,
        error: Result<(), serenity::framework::standard::CommandError>,
    ) {
        if let Err(e) = error {
            if let Some(err) = e.downcast_ref::<wh_core::Error>() {
                match err {
                    wh_core::Error::Error(err) => error!("[{}]{}", cmd_name, err),
                    wh_core::Error::Both { msg, err } => {
                        error!("[{}]{}", cmd_name, err);
                        reply_message!(ctx, message, msg);
                    }
                    wh_core::Error::Message(msg) => {
                        reply_message!(ctx, message, msg);
                    }
                }
            } else {
                error!("[{}] {}", cmd_name, e);
                reply_message!(ctx, message, "Internal Error");
            }
        }
    }

    #[serenity::framework::standard::macros::hook]
    async fn before_hook(
        _: &serenity::client::Context,
        _: &serenity::model::channel::Message,
        cmd_name: &str,
    ) -> bool {
        // debug!("cmd_name: {}", cmd_name);
        true
    }

    #[serenity::framework::standard::macros::hook]
    async fn error_hook(
        ctx: &serenity::client::Context,
        msg: &serenity::model::channel::Message,
        error: serenity::framework::standard::DispatchError,
    ) {
        use serenity::framework::standard::DispatchError::*;
        use serenity::framework::standard::Reason::*;
        match error {
            CheckFailed(check, reason) => match reason {
                User(m) => {
                    reply_message!(ctx, msg, m);
                }
                Log(m) => error!("Check {} failed: {}", check, m),
                UserAndLog { user, log } => {
                    reply_message!(ctx, msg, user);
                    error!("Check {} failed: {}", check, log);
                }
                _ => error!("Check {} failed", check),
            },
            NotEnoughArguments { min, given } => {
                reply_message!(
                    ctx,
                    msg,
                    format!(
                        "The command takes minimum {} arguments but {} were given",
                        min, given
                    )
                );
            }
            TooManyArguments { max, given } => {
                reply_message!(
                    ctx,
                    msg,
                    format!(
                        "The command takes maximum {} arguments but {} were given",
                        max, given
                    )
                );
            }
            _ => {}
        }
    }

    modules!(modules, wh_database, wh_music, wh_points, wh_permission);
    let mut framework = serenity::framework::StandardFramework::new()
        .help(&wh_core::HELP_COMMAND)
        .after(after_hook)
        .on_dispatch_error(error_hook)
        .before(before_hook)
        .configure(|c| {
            c.prefix("wh?");
            c.allow_dm(false);
            c.case_insensitivity(true)
        });

    let mut event_handler = wh_core::event_handler::WhEventHandlerManager::new();
    event_handler.push(WhEventHandler);
    let mut type_map = serenity::prelude::TypeMap::new();
    let mut intent = serenity::client::bridge::gateway::GatewayIntents::empty();
    for module in &modules {
        info!("Loading module \x1b[1;32m{}\x1b[0m", module.module_name);
        for &cmd in module.command_groups {
            framework = framework.group(cmd);
        }
        (module.register_event_handler)(&mut event_handler).await;
        intent = (module.register_intent)(intent);
        (module.register_typemap)(&mut type_map).await;
        (module.register_init)();
    }

    let mut client = serenity::client::Client::builder(std::env::var("WH_DISCORD_BOT_TOKEN").expect(
        "Please use `WH_DISCORD_BOT_TOKEN` environement variable(or .env) with your bot's TOKEN",
    ))
        .framework(framework)
        .event_handler(event_handler)
        .intents(intent)
        .type_map(type_map);
    for module in &modules {
        client = (module.register_builder)(client);
    }

    let client = client.await;

    if let Err(e) = client.as_ref() {
        error!("Error when creating client: {}", e);
    }

    debug!("Start");
    let mut client = client.unwrap();
    match client.start().await {
        Err(e) => error!("Error when starting client: {}", e),
        Ok(_) => {
            info!("Shutdown Client")
        }
    };
    Ok(())
}
