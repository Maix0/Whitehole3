use serenity::{client::Context, model::channel::Message};
struct PointEventHandler;

#[serenity::async_trait]
impl serenity::client::EventHandler for PointEventHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.guild_id.is_none() {
            return;
        }
        let mut lock = ctx.data.write().await;
        let point_data: &mut crate::shared::PointDataList = lock
            .get_mut::<crate::shared::PointKey>()
            .expect("Typemap not initialized");
        let now = std::time::Instant::now();
        if now > point_data.last_sync + crate::shared::SYNC_EVERY {
            debug!("Syncing points with database");
            // TODO: SYNC!
            point_data.last_sync = now;
        }
        let user_data = point_data
            .points
            .get(&(msg.guild_id.unwrap(), msg.author.id));
        if let Some(u) = user_data {
            if now > u.last_time + crate::shared::DELAY_BETWEEN_MESSAGE {}
        } else {
            let db = lock
                .get_mut::<wh_database::shared::DatabaseKey>()
                .expect("Database not in typemap!");
        }
    }
}
