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

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            both_err!(
                "An error occured when leaving the channel",
                format!("Error when leaving a channel: {}", e)
            );
        }
        reply_message!(ctx, msg, "Left Voice Channel");
    } else {
        reply_message!(ctx, msg, "Not in a voice channel");
    }

    Ok(())
}
