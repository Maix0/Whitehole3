use serenity::client::Context;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let sb = songbird::get(ctx).await.unwrap();
    let call_opt = sb.get(msg.guild_id.unwrap());
    let guild = msg.guild(&ctx.cache).await.unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|x| x.channel_id);
    match call_opt {
        Some(call) => {
            if channel_id.map(|c| c.0) == call.lock().await.current_channel().map(|c| c.0) {
                if let Err(e) = call.lock().await.queue().resume() {
                    both_err!(
                        "Error when resuming!",
                        format!("Error when resuming: {}", e)
                    );
                }
            } else {
                message_err!("❌You need to be in the same channel as the bot!");
            }
        }
        None => {
            message_err!("❌ Not connected to a voice channel");
        }
    };
    Ok(())
}
