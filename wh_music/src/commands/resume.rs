use serenity::client::Context;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
/// Resume the music played by the bot
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
                        fluent!(MUSIC_error_resuming),
                        format!(fluent!(MUSIC_LOG_err_resuming), e)
                    );
                }
            } else {
                message_err!(fluent!(MUSIC_not_same_channel));
            }
        }
        None => {
            message_err!(fluent!(MUSIC_voice_not_connected));
        }
    };
    Ok(())
}
