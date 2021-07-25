use serenity::{
    client::Context,
    framework::standard::{CommandResult, Reason},
};
use wh_database::shared::{DatabaseKey, Id};

pub async fn has_permission(
    ctx: &Context,
    userid: u64,
    guildid: u64,
    permission: &str,
) -> Result<bool, Reason> {
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
        "SELECT COUNT(*) FROM permission WHERE userid = $1::int8 AND guildid = $2::int8 AND $3::text = ANY(ids)",
        Id(userid) as _,
        Id(guildid) as _,
        permission
    )
    .fetch_one(database).await;
    if let Err(e) = res.as_ref() {
        return Err(Reason::Log(format!("Database error: {}", e)));
    }
    Ok(res.unwrap().count.unwrap() == 1)
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
            "INSERT INTO permission (guildid, userid, ids)VALUES ($1::int8, $2::int8, $3::text[])",
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
        "SELECT * FROM permission WHERE guildid = $1::int8 AND userid= $2::int8",
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
