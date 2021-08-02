use serenity::client::Context;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[num_args(1)]
/// Remove the music at the given index
pub async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let sb = songbird::get(ctx).await.unwrap();
    let call_opt = sb.get(msg.guild_id.unwrap());
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let index = args.parse::<u16>();
    if index.is_err() {
        message_err!(fluent!(MUSIC_ARG_invalid_number));
    }

    let index = index.unwrap();

    if index == 0 {
        message_err!(fluent!(MUSIC_ARG_invalid_number));
    }
    let index = index - 1;
    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|x| x.channel_id);
    match call_opt {
        Some(call) => {
            if channel_id.map(|c| c.0) == call.lock().await.current_channel().map(|c| c.0) {
                match call.lock().await.queue().dequeue(index as usize) {
                    Some(_) => {
                        reply_message!(ctx, msg, format!(fluent!(MUSIC_remove_item), index + 1));
                    }
                    None => {
                        message_err!(fluent!(MUSIC_ARG_index_oob));
                    }
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
