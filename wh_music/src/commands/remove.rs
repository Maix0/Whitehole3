use serenity::client::Context;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[num_args(1_1)]
/// Remove the music at the given index
pub async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let sb = songbird::get(ctx).await.unwrap();
    let call_opt = sb.get(msg.guild_id.unwrap());
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let index = args.parse::<u16>();
    if index.is_err() {
        message_err!("You need to provide an valid index!");
    }
    let index = index.unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|x| x.channel_id);
    match call_opt {
        Some(call) => {
            if channel_id.map(|c| c.0) == call.lock().await.current_channel().map(|c| c.0) {
                match call.lock().await.queue().dequeue(index as usize) {
                    Some(_) => {
                        reply_message!(ctx, msg, format!("Removed item at index {}", index));
                    }
                    None => {
                        message_err!("❌Index is out of bounds!");
                    }
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
