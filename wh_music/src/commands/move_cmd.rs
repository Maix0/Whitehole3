use serenity::client::Context;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::channel::Message;

#[command("move")]
#[only_in(guilds)]
#[num_args(2)]
#[usage("[src] [dest]")]
#[example("10 1")]
/// This command moves the item at the source index to the destination index, shifting every item after it by one
pub async fn move_cmd(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let sb = songbird::get(ctx).await.unwrap();
    let call_opt = sb.get(msg.guild_id.unwrap());
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let src_index = args.parse::<u16>();
    if src_index.is_err() {
        message_err!(fluent!(MUSIC_ARG_invalid_number));
    }
    let src_index = src_index.unwrap();

    let dest_index = args.parse::<u16>();
    if dest_index.is_err() {
        message_err!(fluent!(MUSIC_ARG_invalid_number));
    }
    let dest_index = dest_index.unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|x| x.channel_id);
    match call_opt {
        Some(call) => {
            if dest_index as usize > call.lock().await.queue().len() {
                message_err!(fluent!(MUSIC_ARG_index_oob))
            }
            if channel_id.map(|c| c.0) == call.lock().await.current_channel().map(|c| c.0) {
                call.lock()
                    .await
                    .queue()
                    .modify_queue(|queue| -> Result<(), wh_core::Error> {
                        let item = queue.remove(src_index as usize);
                        if let Some(item) = item {
                            queue.insert(dest_index as usize, item);
                        } else {
                            message_err!(fluent!(MUSIC_ARG_index_oob));
                        }
                        Ok(())
                    })?;
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
