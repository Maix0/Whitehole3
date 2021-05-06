use serenity::client::EventHandler;

extern crate wh_core;
extern crate wh_database;
extern crate wh_music;

extern crate serenity;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate dotenv;
extern crate fern;
extern crate tokio;

mod event_handler;

struct WhEventHandler;

#[serenity::async_trait]
impl EventHandler for WhEventHandler {
    async fn ready(
        &self,
        _ctx: serenity::client::Context,
        _data_about_bot: serenity::model::gateway::Ready,
    ) {
        info!(
            "Started `{}` on {} guilds",
            _data_about_bot.user.name,
            _data_about_bot.guilds.len()
        );
    }
}

macro_rules! register_event_handler {
    ($base_handler:expr, $($module:ident,)*) => {
        $(match $module::event_handler().await {
            Some(handler) => $base_handler.push(handler),
            None => {},
        };)*
    };
}
macro_rules! register_typemap {
    ($typemap:expr, $($module:ident,)*) => {
        $($module::register_typemap($typemap).await;)*
    };
}

macro_rules! register_builder {
    ($builder:ident, $($module:ident,)*) => {
        $(let $builder = $module::register_builder($builder);)*
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
    fn after_hook<'fut>(
        ctx: &'fut serenity::client::Context,
        message: &'fut serenity::model::channel::Message,
        cmd_name: &'fut str,
        error: Result<(), serenity::framework::standard::CommandError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'fut>> {
        use serenity::FutureExt;
        async move {
            if let Err(e) = error {
                if let Some(err) = e.downcast_ref::<wh_core::Error>() {
                    match err {
                        wh_core::Error::Error(err) => error!("[{}]{}", cmd_name, err),
                        wh_core::Error::Both { msg, err } => {
                            error!("[{}]{}", cmd_name, err);
                            let _ = message
                                .channel(&ctx)
                                .await
                                .unwrap()
                                .guild()
                                .unwrap()
                                .send_message(&ctx.http, |f| f.content(msg))
                                .await
                                .map_err(|e| error!("Error when sending message: {}", e));
                        }
                        wh_core::Error::Message(msg) => {
                            let _ = message
                                .channel(&ctx)
                                .await
                                .unwrap()
                                .guild()
                                .unwrap()
                                .send_message(&ctx.http, |f| f.content(msg))
                                .await
                                .map_err(|e| error!("Error when sending message: {}", e));
                        }
                    }
                } else {
                    error!("[{}]{}", cmd_name, e);
                    let _ = message
                        .channel(&ctx)
                        .await
                        .unwrap()
                        .guild()
                        .unwrap()
                        .send_message(&ctx.http, |f| f.content("Internal Error"))
                        .await
                        .map_err(|e| error!("Error when sending message: {}", e));
                }
            }
        }
        .boxed()
    }

    let framework = serenity::framework::StandardFramework::new()
        .help(&wh_core::HELP_COMMAND)
        .group(&wh_music::MUSIC_GROUP)
        .after(after_hook)
        .configure(|c| c.prefix("wh?"));

    let mut event_handler = event_handler::WhEventHandlerManager::new();
    event_handler.push(WhEventHandler);
    register_event_handler!(event_handler, wh_music, wh_core, wh_database,);

    let mut type_map = serenity::prelude::TypeMap::new();

    register_typemap!(&mut type_map, wh_music, wh_core, wh_database,);

    let client = serenity::client::Client::builder(std::env::var("WH_DISCORD_BOT_TOKEN").expect(
        "Please use `WH_DISCORD_BOT_TOKEN` environement variable(or .env) with your bot's TOKEN",
    ))
    .framework(framework)
    .event_handler(event_handler)
    .intents(serenity::client::bridge::gateway::GatewayIntents::all())
    .type_map(type_map);
    register_builder!(client, wh_core, wh_music, wh_database,);
    let client = client.await;
    if let Err(e) = client.as_ref() {
        error!("Error when creating client: {}", e);
    }
    let mut client = client.unwrap();
    match client.start().await {
        Err(e) => error!("Error when starting client: {}", e),
        Ok(_) => {
            info!("Starting Client")
        }
    };
    Ok(())
}
