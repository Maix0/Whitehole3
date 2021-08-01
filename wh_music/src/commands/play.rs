use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;
use serenity::{client::Context, framework::standard::Args};
use songbird::input::Input;

#[command]
#[only_in(guilds)]
#[usage("[query or url]")]
#[example("https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
#[min_args(1)]
/// Make the bot play the music
/// the query can be a youtube video, a youtube playlist, a simple query or a spotify song/playlist url
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    args.trimmed().unquoted();
    let url = args.parse::<url::Url>();
    let song_url = match url {
        Ok(url) => crate::shared::SongUrl::from_url(url),
        Err(_) => crate::shared::SongUrl::Query(args.remains().unwrap_or("").to_string()),
    };

    let song_query = song_url.into_query().await?;

    let vc = guild.voice_states.get(&ctx.cache.current_user_id().await);
    if vc.is_none() {
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

        let (handler, res) = manager.join(guild_id, connect_to).await;
        res?;
        let meh = crate::shared::MusicEventHandler {
            call: handler.clone(),
        };
        handler.lock().await.add_global_event(
            songbird::events::Event::Track(songbird::events::TrackEvent::End),
            meh,
        );
    }

    let manager = songbird::get(ctx).await.unwrap();

    let call = manager.get(guild.id).unwrap();

    match song_query {
        crate::shared::SongType::SingleUrl(q) => {
            play_yt_url(call, q, ctx, msg, true).await?;
        }
        crate::shared::SongType::MultipleUrl(list) => {
            let mut count = 0;
            for q in list {
                play_yt_url(call.clone(), q, ctx, msg, false).await?;
                count += 1;
            }
            reply_message!(ctx, msg, format!("Added {} song(s) to the queue", count));
        }
        crate::shared::SongType::SingleQuery(q) => {
            play_yt_url(call, format!("ytsearch1:{}", q), ctx, msg, true).await?;
        }
        crate::shared::SongType::MultipleQuery(list) => {
            let mut count = 0;
            let show_addition = list.len() == 1;
            for q in list {
                play_yt_url(
                    call.clone(),
                    format!("ytsearch1:{}", q),
                    ctx,
                    msg,
                    show_addition,
                )
                .await?;
                count += 1;
            }
            reply_message!(ctx, msg, format!("Added {} song(s) to the queue", count));
        }
    }
    Ok(())
}

async fn play_yt_url<U>(
    call: std::sync::Arc<tokio::sync::Mutex<songbird::Call>>,
    url: U,
    ctx: &Context,
    msg: &Message,
    show_addition: bool,
) -> CommandResult
where
    U: AsRef<str> + Send + Sync + Clone + 'static,
{
    match songbird::input::restartable::Restartable::ytdl(url, true).await {
        Ok(y) => {
            if call.lock().await.queue().len() >= crate::shared::MAX_QUEUED_ITEM {
                message_err!("❌There is too many items in the queue!");
            }
            let song: Input = y.into();
            let metadata = crate::shared::TrackMetadata {
                url: song.metadata.source_url.clone(),
                title: song.metadata.title.clone(),
                duration: song.metadata.duration,
                added_by: msg.author.id,
            };
            if show_addition {
                if let Some(u) = metadata.url.as_ref() {
                    reply_message!(ctx, msg, format!("Added {url} to the queue", url = u));
                } else {
                    reply_message!(ctx, msg, "Added the song to the queue");
                }
            }
            let (track, handle) = songbird::tracks::create_player(song);
            handle
                .typemap()
                .write()
                .await
                .insert::<crate::shared::TrackMetadataKey>(metadata);
            let mut call_lock = call.lock().await;
            call_lock.enqueue(track);
            Ok(())
        }
        Err(e) => {
            if let songbird::input::error::Error::Io(_) = &e {
                error_err!("You need to have youtube-dl installed!");
            } else {
                message_err!("❌ No video was found for this song");
            }
        }
    }
}
