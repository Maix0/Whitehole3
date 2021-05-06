extern crate async_trait;
extern crate serenity;
extern crate sqlx;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate dotenv;
extern crate fern;
extern crate once_cell;
extern crate songbird;
extern crate url;

use songbird::SerenityInit;
use std::collections::HashSet;

use once_cell::sync::Lazy;
use serenity::{
    client::{bridge::gateway::GatewayIntents, Context, EventHandler},
    framework::standard::{help_commands, Args, CommandGroup, CommandResult, HelpOptions},
    model::{id::UserId, prelude::Ready},
    prelude::TypeMap,
};
use serenity::{framework::standard::macros::help, Client};

// pub static MEMORY_DB: Lazy<sqlx::Pool<sqlx::Sqlite>> = Lazy::new(|| {
//     sqlx::Pool::connect_lazy("sqlite::memory:")
//         .map_err(|e| error!("DB init: {}", e))
//         .unwrap()
// });

pub static DATABASE: Lazy<sqlx::PgPool> = Lazy::new(|| {
    sqlx::Pool::connect_lazy(
        std::env::var("WH_DATABASE_URL")
            .expect("Use `WH_DATABASE_URL` environment variable to set the database url")
            .as_str(),
    )
    .map_err(|e| error!("DB init: {}", e))
    .unwrap()
});

#[derive(Debug)]
struct WhEventHandler {}

#[serenity::async_trait]
impl EventHandler for WhEventHandler {
    async fn ready(&self, _: Context, bot: Ready) {
        info!(
            "Client `{}` Started. Serving {} guilds",
            bot.user.name,
            bot.guilds.len()
        )
    }
}

mod music;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    logger_setup().expect("Error when setting up logger");

    database_setup()
        .await
        .expect("Error when setting up database");

    bot_launch().await.expect("Error when launching bot");
}

async fn bot_launch() -> Result<(), Box<dyn std::error::Error>> {
    let framework = serenity::framework::StandardFramework::new()
        .help(&HELP_COMMAND)
        .group(&music::MUSIC_GROUP)
        .configure(|c| c.prefix("wh?"));

    let event_handler = WhEventHandler {};

    let mut type_map = TypeMap::new();
    type_map.insert::<music::GuildMusicTypeKey>(music::GuildMusicManager::default());

    let client = Client::builder(std::env::var("WH_DISCORD_BOT_TOKEN").expect(
        "Please use `WH_DISCORD_BOT_TOKEN` environement variable(or .env) with your bot's TOKEN",
    ))
    .framework(framework)
    .event_handler(event_handler)
    .intents(GatewayIntents::all())
    //.type_map(type_map)
    .register_songbird()
    .await;
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

#[help]
async fn help_command(
    context: &Context,
    msg: &serenity::model::channel::Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
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

async fn database_setup() -> Result<(), Box<dyn std::error::Error>> {
    #![allow(unused_variables)]
    let connection = DATABASE.acquire().await?;
    // let mem_connection = MEMORY_DB.acquire().await?;
    // TODO: ENABLE RECONSTRUCTION OF THE DBs
    Ok(())
}
