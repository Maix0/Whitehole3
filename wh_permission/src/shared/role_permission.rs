use once_cell::sync::Lazy;
use serenity::{client::Context, framework::standard::CommandResult};
use wh_database::shared::{DatabaseKey, Id};

const CACHE_SIZE: usize = 100;

type RoleCache = parking_lot::Mutex<
    lru::LruCache<
        u64, /*guilid*/
        std::collections::HashMap<
            String, /*permission*/
            std::collections::HashSet<u64 /*roleid*/>,
        >,
    >,
>;

pub static ROLE_CACHE: Lazy<RoleCache> =
    Lazy::new(|| parking_lot::Mutex::new(lru::LruCache::new(CACHE_SIZE)));

fn check_role_permission_in_cache(guildid: u64, roleid: u64, permission: &str) -> Option<bool> {
    let mut lock = ROLE_CACHE.lock();
    let guild_cache = lock.get(&guildid);
    if let Some(guild) = guild_cache {
        guild.get(permission).map(|roles| roles.contains(&roleid))
    } else {
        None
    }
}

pub async fn get_role_permission(
    ctx: &Context,
    roleid: u64,
    guildid: u64,
) -> Result<Option<RolePermission>, Box<dyn std::error::Error + Send + Sync>> {
    let lock = ctx.data.read().await;
    let db = lock.get::<DatabaseKey>().unwrap();
    let res = query_as!(
        RolePermissionRaw,
        "SELECT * FROM role_permission WHERE roleid = $1::int8 AND guildid = $2::int8",
        Id(roleid) as _,
        Id(guildid) as _
    )
    .fetch_optional(db)
    .await?;
    Ok(res.map(|r| r.into_processed()))
}

pub async fn check_role_permission(
    ctx: &Context,
    guildid: u64,
    roleid: u64,
    permission: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    fetch_role_db(ctx, guildid).await?;
    Ok(check_role_permission_in_cache(guildid, roleid, permission).unwrap_or(false))
}

async fn fetch_role_db(ctx: &Context, guildid: u64) -> CommandResult {
    let is_in_cache = ROLE_CACHE.lock().get(&guildid).is_some();
    if is_in_cache {
        return Ok(());
    }
    role_update_cache_from_db(ctx, guildid).await
}
async fn role_update_cache_from_db(ctx: &Context, guildid: u64) -> CommandResult {
    use serenity::futures::StreamExt;
    let typemap = ctx.data.read().await;

    let db = typemap.get::<DatabaseKey>().unwrap();

    let mut res = query_as!(
        RolePermissionRaw,
        "SELECT * FROM role_permission WHERE guildid = $1::int8",
        Id(guildid) as _
    )
    .fetch(db);
    let mut role_hashmap =
        std::collections::HashMap::<String, std::collections::HashSet<u64>>::with_capacity(
            res.size_hint().1.unwrap_or(10),
        );

    while let Some(row) = res.next().await {
        let row = row?;
        let processed = row.into_processed();
        for perm in processed.ids {
            if let Some(hs) = role_hashmap.get_mut(perm.as_str()) {
                hs.insert(processed.roleid.0);
            } else {
                let mut hs = std::collections::HashSet::new();
                hs.insert(processed.roleid.0);
                role_hashmap.insert(perm, hs);
            }
        }
    }
    role_hashmap.shrink_to_fit();

    ROLE_CACHE.lock().put(guildid, role_hashmap);

    Ok(())
}

pub async fn create_role_permission_if_not_exist(
    ctx: &Context,
    roleid: u64,
    guildid: u64,
) -> CommandResult {
    let typemap = ctx.data.read().await;
    let db = typemap.get::<DatabaseKey>().unwrap();
    let exist = query!(
        "SELECT COUNT(*) FROM role_permission WHERE roleid = $1::int8 and guildid = $2::int8",
        Id(roleid) as _,
        Id(guildid) as _
    )
    .fetch_one(db)
    .await?
    .count
        == Some(1);
    if exist {
        return Ok(());
    }
    query!("INSERT INTO role_permission (roleid, guildid, ids) VALUES ($1::int8, $2::int8, $3::text[])", Id(roleid) as _, Id(guildid) as _, &[][..] ).execute(db).await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct RolePermissionRaw {
    uid: i64,
    guildid: i64,
    roleid: i64,
    ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RolePermission {
    pub uid: i64,
    pub guildid: Id,
    pub roleid: Id,
    pub ids: Vec<String>,
}

impl RolePermissionRaw {
    fn into_processed(self) -> RolePermission {
        RolePermission {
            uid: self.uid,
            ids: self.ids,
            guildid: self.guildid.into(),
            roleid: self.roleid.into(),
        }
    }
}
