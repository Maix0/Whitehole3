use serenity::{
    client::Context,
    framework::standard::{CommandResult, Reason},
};
use wh_database::shared::{DatabaseKey, Id};

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
            match super::role_permission::check_role_permission(ctx, guildid, roleid.0, permission)
                .await
            {
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
    let res = crate::shared::user_permission::has_permission(
        ctx,
        msg,
        msg.author.id.0,
        msg.guild_id.unwrap().0,
        permission,
    )
    .await?;
    if !res {
        return Err(Reason::User(format!(
            "❌You don't have the permission `{}` required to use this command",
            permission
        )));
    }
    Ok(())
}
