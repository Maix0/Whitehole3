pub static BASE_URL: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    dotenv::dotenv().expect("Error with dotenv");
    std::env::var("WH_WEB_SERVER").expect("You need to provide the WH_WEB_SERVER env variable")
});

use image::GenericImageView;
use serenity::{
    client::Context,
    framework::standard::CommandResult,
    model::id::{GuildId, RoleId, UserId},
};

pub struct TimeMapkey;

impl serenity::prelude::TypeMapKey for TimeMapkey {
    type Value = TimeMap;
}

pub struct TimeMap {
    inner: std::collections::HashMap<(GuildId, UserId), std::time::Instant>,
}

const DURATION_BETWEEN_POINTS: std::time::Duration = std::time::Duration::from_secs(45);

impl TimeMap {
    pub fn get_user(&self, guild: GuildId, user: UserId) -> Option<&std::time::Instant> {
        self.inner.get(&(guild, user))
    }
    pub fn is_valid(&self, guild: GuildId, user: UserId) -> bool {
        if let Some(time) = self.get_user(guild, user) {
            time.elapsed() > DURATION_BETWEEN_POINTS
        } else {
            true
        }
    }
    pub fn update(&mut self, guild: GuildId, user: UserId) {
        self.inner.insert((guild, user), std::time::Instant::now());
    }

    pub fn new(capacity: usize) -> Self {
        Self {
            inner: std::collections::HashMap::with_capacity(capacity),
        }
    }
}

// ------------------------------------------------------------------------------------

pub async fn create_user_if_not_exist(
    ctx: &Context,
    userid: u64,
    guildid: u64,
) -> CommandResult<UserPoint> {
    let mut usr = get_user(ctx, userid, guildid).await?;
    if usr.is_none() {
        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
        usr = query_as!(
            UserPointRaw,
            "INSERT INTO user_points (userid, guildid, points) VALUES ($1::int8, $2::int8, 0) RETURNING *",
            wh_database::shared::Id(userid) as _,
            wh_database::shared::Id(guildid) as _,
        )
        .fetch_optional(db)
        .await?.map(|r| r.into_processed());
    }
    Ok(usr.unwrap())
}

pub async fn get_user(
    ctx: &Context,
    userid: u64,
    guildid: u64,
) -> CommandResult<Option<UserPoint>> {
    let lock = ctx.data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
    let res = query_as!(
        UserPointRaw,
        "SELECT * from user_points WHERE userid=$1::int8 and guildid= $2::int8",
        wh_database::shared::Id(userid) as _,
        wh_database::shared::Id(guildid) as _
    )
    .fetch_optional(db)
    .await?;

    Ok(res.map(|r| r.into_processed()))
}

struct UserPointRaw {
    uid: i64,
    userid: i64,
    guildid: i64,
    points: i64,
}

pub struct UserPoint {
    pub uid: i64,
    pub userid: wh_database::shared::Id,
    pub guildid: wh_database::shared::Id,
    pub points: i64,
}

impl UserPointRaw {
    fn into_processed(self) -> UserPoint {
        UserPoint {
            uid: self.uid,
            userid: self.userid.into(),
            guildid: self.guildid.into(),
            points: self.points,
        }
    }
}

// ------------------------------------------------

struct RolePointsRaw {
    uid: i64,
    roleid: i64,
    guildid: i64,
    points: i64,
}

pub struct RolePoints {
    pub uid: i64,
    pub roleid: wh_database::shared::Id,
    pub guildid: wh_database::shared::Id,
    pub points: i64,
}

impl RolePointsRaw {
    fn into_processed(self) -> RolePoints {
        RolePoints {
            uid: self.uid,
            roleid: self.roleid.into(),
            guildid: self.guildid.into(),
            points: self.points,
        }
    }
}

pub async fn get_role_points(
    ctx: &Context,
    guildid: u64,
    roleid: u64,
) -> CommandResult<Option<RolePoints>> {
    let lock = ctx.data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

    let res = query_as!(
        RolePointsRaw,
        "SELECT * FROM role_points WHERE roleid = $1::int8 AND guildid = $2::int8",
        wh_database::shared::Id(roleid) as _,
        wh_database::shared::Id(guildid) as _
    )
    .fetch_optional(db)
    .await?;

    Ok(res.map(|r| r.into_processed()))
}

// ----------------------------------------------------------

use serenity::model::channel::Message;
use std::collections::{HashMap, HashSet};
pub async fn get_all_role_for_user(
    ctx: &Context,
    userid: u64,
    guildid: u64,
) -> CommandResult<std::collections::HashSet<RoleId>> {
    let lock = ctx.data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

    let res = query!(
        "
        SELECT  array_agg(roleid) from role_points WHERE points <= 
        (SELECT points from user_points WHERE userid = $1::int8 AND guildid = $2::int8)
        AND guildid = $2::int8 
        GROUP BY points 
        ORDER BY points DESC
        ",
        wh_database::shared::Id(userid) as _,
        wh_database::shared::Id(guildid) as _,
    )
    .fetch_one(db)
    .await?;
    let arr = res.array_agg.unwrap_or_else(Vec::new);
    let set: HashSet<RoleId, _> = arr
        .into_iter()
        .map(|id| RoleId(unsafe { std::mem::transmute::<i64, u64>(id) }))
        .collect();
    Ok(set)
}

pub async fn handle_user_message(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.is_private() || msg.author.bot {
        return Ok(());
    }
    let lock = ctx.data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
    let timemap = lock.get::<crate::shared::TimeMapkey>().unwrap();
    if timemap.is_valid(msg.guild_id.unwrap(), msg.author.id) {
        let _ =
            crate::shared::create_user_if_not_exist(&ctx, msg.author.id.0, msg.guild_id.unwrap().0)
                .await?;

        let _ = query!("UPDATE user_points SET points = points + random_between(10,20) WHERE userid = $1::int8 and guildid = $2::int8", wh_database::shared::Id(msg.author.id.0) as _, 
        wh_database::shared::Id(msg.guild_id.unwrap().0) as _).execute(db).await?;

        // drop((db, timemap));
        drop(lock);
        let mut lock = ctx.data.write().await;
        let timemap = lock.get_mut::<crate::shared::TimeMapkey>().unwrap();
        timemap.update(msg.guild_id.unwrap(), msg.author.id);
        drop(lock);
        let res =
            crate::shared::get_all_role_for_user(&ctx, msg.author.id.0, msg.guild_id.unwrap().0)
                .await?;

        let mut member = msg.member(&ctx).await?;

        let roles: std::collections::HashSet<_> = member.roles.clone().into_iter().collect();

        let diff = res.difference(&roles).cloned().collect::<Vec<_>>();
        let _ = member.add_roles(&ctx, &diff).await?;
    }
    Ok(())
}

/*
/---------------------------------------------------------------------------\
|  ____          _                    ___                                   |
| / ___|   _ ___| |_ ___  _ __ ___   |_ _|_ __ ___   __ _  __ _  ___  ___   |
|| |  | | | / __| __/ _ \| '_ ` _ \   | || '_ ` _ \ / _` |/ _` |/ _ \/ __|  |
|| |__| |_| \__ \ || (_) | | | | | |  | || | | | | | (_| | (_| |  __/\__ \  |
| \____\__,_|___/\__\___/|_| |_| |_| |___|_| |_| |_|\__,_|\__, |\___||___/  |
|                                                         |___/             |
\---------------------------------------------------------------------------/
*/

pub async fn add_rank_image_file(
    guildid: u64,
    userid: u64,
    image_data: &[u8],
) -> CommandResult<String> {
    let img = image::load_from_memory(image_data)?;

    let image_view =
        tokio::spawn(
            async move { img.resize_exact(500, 200, image::imageops::FilterType::Lanczos3) },
        )
        .await
        .unwrap();

    let mut data = std::fs::OpenOptions::new();
    data.write(true).truncate(true).create(true);

    let out_file = format!(
        "{base}/images/rank/{guild}_{user}.png",
        base = std::env::var("WH_BASE_FS").expect("Must set WH_BASE_FS"),
        guild = guildid,
        user = userid
    );

    let mut data = data.open(&out_file)?;

    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut data,
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::Avg,
    );
    encoder.encode(
        image_view.as_bytes(),
        image_view.width(),
        image_view.height(),
        image_view.color(),
    )?;

    Ok(out_file)
}
