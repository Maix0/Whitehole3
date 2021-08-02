use serenity::client::Context;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[usage("")]
#[num_args(0)]
/// Make the bot leave his voice channel and clearing the queue
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await.unwrap().clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            both_err!(
                fluent!(MUSIC_error_leaving_channel),
                format!(fluent!(MUSIC_LOG_err_leaving_channel), e)
            );
        }
        reply_message!(ctx, msg, fluent!(MUSIC_left_voice_channel));
    } else {
        reply_message!(ctx, msg, fluent!(MUSIC_voice_not_connected));
    }

    Ok(())
}
