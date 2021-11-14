use serenity::{client::Context, model::channel::Message};
pub struct PointEventHandler;

#[serenity::async_trait]
impl serenity::client::EventHandler for PointEventHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Err(e) = crate::shared::handle_user_message(&ctx, &msg).await {
            error!("{}", e);
        }
    }

    async fn guild_member_addition(
        &self,
        ctx: Context,
        guild_id: serenity::model::id::GuildId,
        new_member: serenity::model::guild::Member,
    ) {
        if let Err(e) = crate::shared::handle_join_event(ctx, guild_id, new_member).await {
            error!("{}", e);
        }
    }
}
