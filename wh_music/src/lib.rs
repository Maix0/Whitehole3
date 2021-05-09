extern crate serenity;
extern crate songbird;
#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate wh_core;
extern crate serenity_utils;
extern crate ureq;

use serenity::framework::standard::{macros::*, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::{client::Context, framework::standard::Args};

pub async fn register_typemap(_: &mut serenity::prelude::TypeMap) {}

pub async fn event_handler() -> Option<wh_core::EmptyEventHandler> {
    None
}

pub fn register_builder(
    client: serenity::client::ClientBuilder<'_>,
) -> serenity::client::ClientBuilder<'_> {
    use songbird::SerenityInit;
    client.register_songbird()
}

#[derive(Clone, Debug)]
pub enum SongUrl {
    YoutubeVideo(url::Url),
    Spotify(url::Url),
    Query(String),
}

/*
fn get_yt_url<S: AsRef<str>>(query: S) -> CommandResult<Option<url::Url>> {
    static mut API_KEY: Option<String> = None;
    if unsafe { API_KEY.is_none() } {
        unsafe {
            API_KEY = Some(
                std::env::var("WH_GOOGLE_API_KEY")
                    .expect("You need to set the `WH_GOOGLE_API_KEY` environement variable"),
            );
        }
    }

    let query = query.as_ref();
    let query_params = [
        ("part", "snippet"),
        ("order", "relevance"),
        ("q", query.as_ref()),
        ("type", "video"),
        ("key", unsafe { API_KEY.as_ref().unwrap().as_str() }),
    ];
    let url = url::Url::parse_with_params(
        "https://youtube.googleapis.com/youtube/v3/search",
        &query_params,
    )
    .unwrap();
    let req = ureq::get(url.as_str()).call()?;
    let json: ureq::SerdeValue = req.into_json()?;
    let val = json.pointer("/items/0/id/videoId");
    if let Some(ureq::SerdeValue::String(s)) = val {
        return Ok(Some(
            url::Url::parse(&format!("https://youtube.com/watch?v={}", s)).unwrap(),
        ));
    } else {
        Ok(None)
    }
}
*/
#[derive(Clone, Debug)]
enum Query {
    Single(String),
    Multiple(Vec<String>),
}

impl SongUrl {
    fn from_url(url: url::Url) -> Option<Self> {
        match url.host() {
            Some(url::Host::Domain(u)) => match u {
                "youtube.com" | "youtu.be" | "www.youtube.com" | "www.youtu.be" => match url.path()
                {
                    "/watch" => Some(Self::YoutubeVideo(url)),
                    "/playlist" => None,
                    _ => Some(Self::Query(url.to_string())),
                },
                "spotify.com" => Some(Self::Spotify(url)),
                _ => Some(Self::Query(url.to_string())),
            },
            _ => Some(Self::Query(url.to_string())),
        }
    }
    async fn into_query(self) -> Result<Query, CommandError> {
        Ok(match self {
            Self::YoutubeVideo(s) => Query::Single(s.to_string()),
            Self::Query(s) => Query::Single(s),
            Self::Spotify(s) => {
                todo!()
            }
        })
    }
}

#[group]
#[only_in(guild)]
#[commands(join, play)]
pub struct Music;

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
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

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    args.trimmed().unquoted();
    let url = args.single::<url::Url>();
    let song_url = match url {
        Ok(url) => SongUrl::from_url(url),
        Err(_) => args.remains().map(|s| SongUrl::Query(s.to_string())),
    };
    if song_url.is_none() {
        message_err!("Please input valid url or query")
    }
    let song_url = song_url.unwrap();
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

        let (_handler, res) = manager.join(guild_id, connect_to).await;
        res?;
    }

    let manager = songbird::get(ctx).await.unwrap();

    let call = manager.get(guild.id).unwrap();
    let mut call_lock = call.lock().await;
    match song_query {
        Query::Single(q) => {
            let song =
                songbird::input::restartable::Restartable::ytdl_search(q.as_str(), true).await;
            if let Ok(song) = song {
                call_lock.enqueue_source(song.into());
            } else {
                if let Ok(y) = songbird::input::restartable::Restartable::ytdl(q, true).await {
                    call_lock.enqueue_source(y.into());
                } else {
                    message_err!("❌ No video was found for this song")
                }
            }
        }
        Query::Multiple(list) => {
            //TODO: Handle playlist
            todo!()
        }
    }
    std::mem::drop(call_lock);
    Ok(())
}
