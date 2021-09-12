pub use serde::{de::DeserializeOwned, Serialize};

pub trait Config: Serialize + DeserializeOwned {
    const KEY: &'static str;
}

pub struct ConfigGuard<T: Config> {
    db: sqlx::pool::PoolConnection<sqlx::Postgres>,
    guildid: u64,
    data: T,
}

use std::fmt::Debug;
impl<T: Config + Debug + ?Sized> Debug for ConfigGuard<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

use std::fmt::Display;
impl<T: Config + Display + ?Sized> Display for ConfigGuard<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
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
        panic!("This struct must be discareded with the function \"set_config\" and not dropped")
    }
}

type AllResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn _get_config<T: Config>(
    database: &sqlx::PgPool,
    guildid: u64,
) -> AllResult<(Option<T>, sqlx::pool::PoolConnection<sqlx::Postgres>)> {
    let mut conn = database.acquire().await?;
    let res = query!(
        "SELECT * FROM get_config($1::int8, $2::varchar)",
        wh_database::shared::Id(guildid) as _,
        <T as Config>::KEY,
    )
    .fetch_optional(&mut conn)
    .await?;
    Ok(match res {
        Some(e) => match e.get_config {
            Some(e) => {
                let val: Result<T, _> = serde_json::value::from_value(e);
                if let Err(e) = &val {
                    error!(
                        "Error when deserializing config `{}`: {}",
                        <T as Config>::KEY,
                        e
                    );
                    (None, conn)
                } else {
                    (Some(val.unwrap()), conn)
                }
            }
            None => (None, conn),
        },
        None => (None, conn),
    })
}

pub async fn get_config_or_default<T: Config + Default>(
    database: &sqlx::PgPool,
    guildid: u64,
) -> AllResult<ConfigGuard<T>> {
    let (data, conn) = _get_config::<T>(database, guildid).await?;

    Ok(ConfigGuard {
        guildid,
        db: conn,
        data: data.unwrap_or_default(),
    })
}

pub async fn get_config<T: Config>(
    database: &sqlx::PgPool,
    guildid: u64,
) -> AllResult<Option<ConfigGuard<T>>> {
    let (data, conn) = _get_config::<T>(database, guildid).await?;

    Ok(data.map(|c| ConfigGuard {
        guildid,
        db: conn,
        data: c,
    }))
}

pub async fn set_config<T: Config>(guard: ConfigGuard<T>) -> AllResult<()> {
    let mut guard = guard;
    query!(
        "SELECT * FROM set_config($1::int8, $2::varchar, $3::jsonb)",
        wh_database::shared::Id(guard.guildid) as _,
        <T as Config>::KEY,
        serde_json::value::to_value(&guard.data)? as _,
    )
    .execute(&mut guard.db)
    .await?;
    // SAFETY: this is safe because we forget the `guard` juste after this so nobody can use dropped memory
    unsafe {
        std::ptr::drop_in_place(&mut guard.data as *mut _);
        std::ptr::drop_in_place(&mut guard.db as *mut _);
        std::ptr::drop_in_place(&mut guard.guildid as *mut _);
    };
    std::mem::forget(guard);
    Ok(())
}
