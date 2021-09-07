pub use serde::{de::DeserializeOwned, Serialize};

pub trait Config: Serialize + DeserializeOwned {
    const KEY: &'static str;
}

pub struct ConfigGuard<T: Config> {
    db: sqlx::pool::PoolConnection<sqlx::Postgres>,
    guildid: u64,
    data: T,
}

impl<T: Config> std::ops::Deref for ConfigGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Config> std::ops::DerefMut for ConfigGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: Config> Drop for ConfigGuard<T> {
    fn drop(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { set_config(self.db, self.guildid, self.data) });
        error!("Please use set_config() function for destructing a config")
    }
}

type AllResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn get_config<T: Config>(database: &sqlx::PgPool, guildid: u64) -> AllResult<Option<T>> {
    let mut conn = database.acquire().await?;
    let res = query!(
        "SELECT * FROM get_config($1::int8, $2::varchar)",
        wh_database::shared::Id(guildid) as _,
        <T as Config>::KEY,
    )
    .fetch_optional(&mut conn)
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

async fn set_config<T: Config>(guard: &mut ConfigGuard<T>) -> AllResult<()> {
    query!(
        "UPDATE guild_options SET data = $1::jsonb WHERE guildid = $2::int8",
        serde_json::value::to_value(&guard.data)? as _,
        wh_database::shared::Id(guard.guildid) as _
    )
    .execute(&mut guard.db)
    .await?;
    Ok(())
}
