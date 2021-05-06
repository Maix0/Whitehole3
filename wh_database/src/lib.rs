use serenity::prelude::TypeMapKey;

extern crate serenity;
extern crate sqlx;
extern crate wh_core;

pub struct DatabaseKey;

impl TypeMapKey for DatabaseKey {
    type Value = sqlx::PgPool;
}

pub async fn event_handler() -> Option<wh_core::EmptyEventHandler> {
    None
}

pub async fn register_typemap(tm: &mut serenity::prelude::TypeMap) {
    let db = sqlx::PgPool::connect(
        std::env::var("WH_DATABASE_URL")
            .expect("Use `WH_DATABASE_URL` environment variable to set the database url")
            .as_str(),
    )
    .await
    .expect("Error when connection to database");

    tm.insert::<DatabaseKey>(db);
}

pub fn register_builder(
    client: serenity::client::ClientBuilder<'_>,
) -> serenity::client::ClientBuilder<'_> {
    client
}
