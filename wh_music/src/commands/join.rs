use serenity::client::Context;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[usage("")]
#[num_args(0_0)]
/// Make the bot join your voice channel
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|x| x.channel_id);

    let connect_to = match channel_id {
        None => {
            message_err!("You need to be connected in a voice channel to use this command")
        }
        Some(vc) => vc,
    };

    let manager = songbird::get(ctx).await.unwrap();

    let (_handler, res) = manager.join(guild_id, connect_to).await;

    res?;
    Ok(())
}
