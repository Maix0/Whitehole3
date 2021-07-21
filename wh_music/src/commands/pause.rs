use serenity::client::Context;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[usage("")]
#[num_args(0_0)]
/// Pause the bot music
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
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
                if let Err(e) = call.lock().await.queue().pause() {
                    both_err!("Error when pausing!", format!("Error when pausing: {}", e));
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
