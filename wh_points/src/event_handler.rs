use serenity::{client::Context, model::channel::Message};
pub struct PointEventHandler;

#[serenity::async_trait]
impl serenity::client::EventHandler for PointEventHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Err(e) = crate::shared::handle_user_message(&ctx, &msg).await {
            error!("{}", e);
        }
    }
}
