#[macro_use]
extern crate rocket;
extern crate serenity;
extern crate wh_core;
#[macro_use]
extern crate log;
#[macro_use]
extern crate sqlx;
extern crate base64;
extern crate reqwest;
extern crate resvg;
extern crate tiny_skia;
extern crate usvg;

use rocket::tokio;
use serenity::prelude::TypeMap;
use std::sync::Arc;

use tokio::sync::RwLock;

mod api;

pub type Data = Arc<RwLock<TypeMap>>;
pub type CacheHttp = Arc<serenity::CacheAndHttp>;
pub type ShardManager = Arc<tokio::sync::Mutex<serenity::client::bridge::gateway::ShardManager>>;
pub async fn set_webserver(typemap: Data, cache_http: CacheHttp, shard: ShardManager) {
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
    let shutdown = res.shutdown();
    tokio::spawn(async move {
        shutdown.await;
        shard.lock().await.shutdown_all().await;
    });

    let res = res.launch().await;
    if let Err(e) = &res {
        error!("{}", e);
    }
}
