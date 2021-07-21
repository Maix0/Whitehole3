//use serenity::{client::Context, model::channel::Message};
struct PointEventHandler;

#[serenity::async_trait]
impl serenity::client::EventHandler for PointEventHandler {
    // async fn message(&self, ctx: Context, msg: Message) {
    //     if msg.guild_id.is_none() {
    //         return;
    //     }
    //     let mut lock = ctx.data.write().await;
    //     let point_data: &mut crate::shared::PointDataList = lock
    //         .get_mut::<crate::shared::PointKey>()
    //         .expect("Typemap not initialized");
    //     let now = std::time::Instant::now();
    //     if now > point_data.last_sync + crate::shared::SYNC_EVERY {
    //         debug!("Syncing points with database");
    //         // TODO: SYNC!
    //         point_data.last_sync = now;
    //     }
    //     let user_data = point_data
    //         .points
    //         .get(&(msg.guild_id.unwrap(), msg.author.id));
    //     if let Some(u) = user_data {
    //         if now > u.last_time + crate::shared::DELAY_BETWEEN_MESSAGE {}
    //     } else {
    //         let mut db = lock
    //             .get::<wh_database::shared::DatabaseKey>()
    //             .expect("Database not in typemap!");
    //         let (guildid, userid) = (
    //             wh_database::shared::Id(msg.guild_id.unwrap().0),
    //             wh_database::shared::Id(msg.author.id.0),
    //         );
    //         let data = sqlx::query_as!(
    //             r#"SELECT * from "public"."points" WHERE guildid = $1 AND userid = $2"#,
    //             guildid,
    //             userid
    //         )
    //         .fetch_optional(db)
    //         .await;
    //         if let Err(e) = data {
    //             error!("Internal Database Error: {e}", e);
    //             return;
    //         }
    //         let data = data.unwrap();
    //         match data {
    //             None => {}
    //             Some(d) => {}
    //         }
    //     }
    // }
}
