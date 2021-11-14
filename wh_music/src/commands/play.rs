use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;
use serenity::{client::Context, framework::standard::Args};
#[command]
#[only_in(guilds)]
#[aliases("p")]
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
        super::join(ctx, msg, args).await?;
    }

    let manager = songbird::get(ctx).await.unwrap();

    let call = manager.get(guild.id).unwrap();

    match song_query {
        crate::shared::SongType::SingleUrl(q) => {
            crate::shared::play_yt_url(call, q, ctx, msg, true).await?;
        }
        crate::shared::SongType::MultipleUrl(list) => {
            let mut count = 0;
            for q in list {
                crate::shared::play_yt_url(call.clone(), q, ctx, msg, false).await?;
                count += 1;
            }
            reply_message!(
                ctx,
                msg,
                format!(fluent!(MUSIC_add_to_queue_multiple), count)
            );
        }
        crate::shared::SongType::SingleQuery(q) => {
            crate::shared::play_yt_url(call, format!("ytsearch1:{}", q), ctx, msg, true).await?;
        }
        crate::shared::SongType::MultipleQuery(list) => {
            let mut count = 0;
            let show_addition = list.len() == 1;
            for q in list {
                crate::shared::play_yt_url(
                    call.clone(),
                    format!("ytsearch1:{}", q),
                    ctx,
                    msg,
                    show_addition,
                )
                .await?;
                count += 1;
            }
            reply_message!(
                ctx,
                msg,
                format!(fluent!(MUSIC_add_to_queue_multiple), count)
            );
        }
    }
    Ok(())
}
