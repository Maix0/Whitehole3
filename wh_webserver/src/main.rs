#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;
#[macro_use]
extern crate sqlx;

extern crate base64;
extern crate dotenv;
extern crate reqwest;
extern crate resvg;
extern crate serenity;
extern crate tiny_skia;
extern crate usvg;
extern crate wh_core;

use rocket::tokio;
use serenity::prelude::TypeMap;
use std::sync::Arc;

use tokio::sync::RwLock;

mod api;

pub type Data = Arc<RwLock<TypeMap>>;
pub type CacheHttp = Arc<serenity::CacheAndHttp>;
pub async fn run_webserver(typemap: Data, cache_http: CacheHttp) {
    let mut config = rocket::config::Config::default();
    config.secret_key = rocket::config::SecretKey::from(&include_bytes!("secretkey")[..]);
    config.port = 9955;
    let res = rocket::custom(config)
        .manage(typemap)
        .manage(cache_http)
        .mount("/api", api::routes())
        .ignite()
        .await;
    if let Err(e) = &res {
        error!("{}", e);
    }
    let res = res.unwrap();

    let res = res.launch().await;
    if let Err(e) = &res {
        error!("{}", e);
    }
}

#[rocket::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let client = serenity::prelude::Client::builder(std::env::var("WH_DISCORD_BOT_TOKEN").unwrap());
    let client = client.framework(serenity::framework::StandardFramework::new());
    let client = client.await;

    if let Err(e) = &client {
        error!("Error when discord starting client: {}", e);
    }
    let client = client.unwrap();

    let typemap = client.data.clone();
    let cache_http = client.cache_and_http.clone();

    let db = sqlx::PgPool::connect(
        std::env::var("DATABASE_URL")
            .expect("Use `DATABASE_URL` environment variable to set the database url")
            .as_str(),
    )
    .await
    .expect("Error when connection to database");

    sqlx::migrate!("../migrations")
        .run(&db)
        .await
        .expect("Error when runnings migrations");
    client
        .data
        .write()
        .await
        .insert::<wh_database::shared::DatabaseKey>(db);
    info!("starting");
    
    run_webserver(typemap, cache_http).await;
}
