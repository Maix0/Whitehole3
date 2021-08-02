use serenity::client::Context;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[usage("")]
#[num_args(0)]
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
            message_err!(fluent!(MUSIC_need_voice_channel))
        }
        Some(vc) => vc,
    };

    let manager = songbird::get(ctx).await.unwrap();

    let (handler, res) = manager.join(guild_id, connect_to).await;
    res?;
    let meh = crate::shared::MusicEventHandler {
        call: handler.clone(),
    };
    handler.lock().await.add_global_event(
        songbird::events::Event::Track(songbird::events::TrackEvent::End),
        meh,
    );
    Ok(())
}
