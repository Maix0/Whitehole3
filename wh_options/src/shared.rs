pub use serde::{de::DeserializeOwned, Serialize};

pub trait Config: Serialize + DeserializeOwned {
    const KEY: &'static str;
}

type AllResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn get_config<T: Config>(database: &sqlx::PgPool, guildid: u64) -> AllResult<Option<T>> {
    let res = query!(
        "SELECT data FROM guild_options WHERE key = $1::varchar(64) AND guildid = $2::int8",
        <T as Config>::KEY,
        wh_database::shared::Id(guildid) as _,
    )
    .fetch_optional(database)
    .await?;
    Ok(match res {
        Some(e) => {
            let val: Result<T, _> = serde_json::value::from_value(e.data);
            if let Err(e) = &val {
                error!(
                    "Error when deserializing config `{}`: {}",
                    <T as Config>::KEY,
                    e
                );
                None
            } else {
                Some(val.unwrap())
            }
        }
        None => None,
    })
}

pub async fn get_config_or_default<T: Config + Default>(
    database: &sqlx::PgPool,
    guildid: u64,
) -> AllResult<T> {
    get_config::<T>(database, guildid)
        .await
        .map(|o| o.unwrap_or_default())
}

pub async fn set_config<T: Config>(database: &sqlx::PgPool, guildid: u64, val: T) -> AllResult<()> {
    query!(
        "UPDATE guild_options SET data = $1::jsonb WHERE guildid = $2::int8",
        serde_json::value::to_value(val)? as _,
        wh_database::shared::Id(guildid) as _
    )
    .execute(database)
    .await?;
    Ok(())
}
