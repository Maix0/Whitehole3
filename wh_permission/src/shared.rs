use once_cell::sync::Lazy;
use serenity::{
    client::Context,
    framework::standard::{CommandResult, Reason},
};
use wh_database::shared::{DatabaseKey, Id};

const CACHE_SIZE: usize = 100;

pub static ROLE_CACHE: Lazy<
    parking_lot::Mutex<
        lru::LruCache<
            u64, /*guilid*/
            std::collections::HashMap<
                String, /*permission*/
                std::collections::HashSet<u64 /*roleid*/>,
            >,
        >,
    >,
> = Lazy::new(|| parking_lot::Mutex::new(lru::LruCache::new(CACHE_SIZE)));

fn check_role_permission_in_cache(guildid: u64, roleid: u64, permission: &str) -> Option<bool> {
    let mut lock = ROLE_CACHE.lock();
    let guild_cache = lock.get(&guildid);
    if let Some(guild) = guild_cache {
        guild.get(permission).map(|roles| roles.contains(&roleid))
    } else {
        return None;
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

async fn check_role_permission(
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

static mut PERMISSIONS: Vec<&'static str> = Vec::new();

pub fn add_permission(perms: &[&'static str]) {
    for p in perms {
        static_set_permission(p);
    }
}
pub fn static_set_permission(perm: &'static str) {
    unsafe { PERMISSIONS.push(perm) };
}

pub fn static_get_permission() -> &'static Vec<&'static str> {
    unsafe { &PERMISSIONS }
}

pub async fn has_permission(
    ctx: &Context,
    msg: &Message,
    userid: u64,
    guildid: u64,
    permission: &str,
) -> Result<bool, Reason> {
    if !static_get_permission().contains(&permission) {
        error!("You need to register the permission `{}` with the wh_permission::add_permission function", permission);
    }
    let lock = ctx.data.read().await;
    let database = lock.get::<DatabaseKey>().unwrap();
    let r = create_permission_if_not_exists(ctx, userid, guildid).await;
    if let Err(e) = &r {
        return Err(Reason::UserAndLog {
            user: "Internal Error".into(),
            log: format!("Database Error when fetching permission: {}", e),
        });
    }
    r.unwrap();
    let res = query!(
        "SELECT COUNT(*) FROM user_permission WHERE userid = $1::int8 AND guildid = $2::int8 AND $3::text = ANY(ids)",
        Id(userid) as _,
        Id(guildid) as _,
        permission
    )
    .fetch_one(database).await;
    if let Err(e) = res.as_ref() {
        return Err(Reason::Log(format!("Database error: {}", e)));
    }
    let mut role_perm = false;
    for roleid in &msg.member(&ctx.http).await.unwrap().roles {
        role_perm = role_perm || {
            match check_role_permission(ctx, guildid, roleid.0, permission).await {
                Ok(b) => b,
                Err(e) => return Err(Reason::Log(format!("Database error: {}", e))),
            }
        };
        if role_perm {
            break;
        }
    }
    Ok(res.unwrap().count.unwrap() == 1 || role_perm)
}

pub async fn create_permission_if_not_exists(
    ctx: &Context,
    userid: u64,
    guildid: u64,
) -> CommandResult {
    let lock = ctx.data.read().await;
    let db = lock.get::<DatabaseKey>().unwrap();
    let user = get_permission(ctx, userid, guildid).await?;
    if user.is_none() {
        query!(
            "INSERT INTO user_permission (guildid, userid, ids)VALUES ($1::int8, $2::int8, $3::text[])",
            Id(guildid) as _,
            Id(userid) as _,
            &[][..]
        )
        .execute(db)
        .await?;
    }
    Ok(())
}

pub async fn get_permission(
    ctx: &Context,
    userid: u64,
    guildid: u64,
) -> Result<Option<UserPermission>, Box<dyn std::error::Error + Send + Sync>> {
    let lock = ctx.data.read().await;
    let db = lock.get::<DatabaseKey>().unwrap();
    let res = query_as!(
        UserPermissionRaw,
        "SELECT * FROM user_permission WHERE guildid = $1::int8 AND userid= $2::int8",
        Id(guildid) as _,
        Id(userid) as _
    )
    .fetch_optional(db)
    .await?;

    Ok(res.map(|u| u.into_processed()))
}

struct UserPermissionRaw {
    uid: i64,
    guildid: i64,
    userid: i64,
    ids: Vec<String>,
}
impl UserPermissionRaw {
    fn into_processed(self) -> UserPermission {
        UserPermission {
            uid: self.uid,
            ids: self.ids,
            guildid: self.guildid.into(),
            userid: self.userid.into(),
        }
    }
}
pub struct UserPermission {
    pub uid: i64,
    pub guildid: wh_database::shared::Id,
    pub userid: wh_database::shared::Id,
    pub ids: Vec<String>,
}
use serenity::framework::standard::macros::hook;
use serenity::model::channel::Message;

#[hook]
pub async fn check_permission(
    ctx: &Context,
    msg: &Message,
    permission: &str,
) -> Result<(), Reason> {
    let res = crate::shared::has_permission(
        ctx,
        msg,
        msg.author.id.0,
        msg.guild_id.unwrap().0,
        permission,
    )
    .await?;
    if !res {
        return Err(Reason::User(format!(
            "❌You don't have the permission `{}` to use this command",
            permission
        )));
    }
    Ok(())
}

#[macro_export]
macro_rules! check_permission {
    ($struct_name:ident, $permission:literal) => {
        const $struct_name: serenity::framework::standard::Check =
            serenity::framework::standard::Check {
                function: |ctx, msg, _, _| {
                    wh_permission::shared::check_permission(ctx, msg, $permission)
                },
                name: $permission,
                display_in_help: true,
                check_in_help: true,
            };
    };
}
