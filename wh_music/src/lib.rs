extern crate serenity;
extern crate songbird;
#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate wh_core;
extern crate chrono;
extern crate serenity_utils;
extern crate ureq;
#[macro_use]
extern crate serde;

use serenity::{
    builder::{CreateEmbed, CreateMessage},
    model::{channel::Message, id::UserId},
};
use serenity::{client::Context, framework::standard::Args};
use serenity::{
    framework::standard::{macros::*, CommandError, CommandResult},
    prelude::TypeMapKey,
};
use serenity_utils::prelude::MenuOptions;
use songbird::input::Input;

pub async fn register_typemap(_: &mut serenity::prelude::TypeMap) {}

pub async fn register_event_handler(_: &mut wh_core::event_handler::WhEventHandlerManager) {}

pub fn register_builder(
    client: serenity::client::ClientBuilder<'_>,
) -> serenity::client::ClientBuilder<'_> {
    use songbird::SerenityInit;
    client.register_songbird()
}

#[derive(Clone, Debug)]
pub enum SongUrl {
    YoutubeVideo(url::Url),
    YoutubePlaylist(url::Url),
    Spotify(url::Url),
    Query(String),
}
#[derive(Clone, Debug)]
struct TrackMetadataKey;

impl TypeMapKey for TrackMetadataKey {
    type Value = TrackMetadata;
}

#[derive(Clone, Debug)]
struct TrackMetadata {
    url: Option<String>,
    duration: Option<std::time::Duration>,
    title: Option<String>,
    added_by: UserId,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlaylistItems {
    #[serde(alias = "nextPageToken")]
    next_page_token: Option<String>,
    #[serde(alias = "pageInfo")]
    page_info: PageInfo,
    items: Vec<Items>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PageInfo {
    #[serde(alias = "totalResults")]
    total_results: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Items {
    #[serde(alias = "snippet.resourceId.videoId")]
    video_id: String,
}

fn get_yt_playlist_urls<S: AsRef<str>>(query: S) -> CommandResult<Vec<url::Url>> {
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
    let q_url = url::Url::parse(query)?;
    let mut pairs = q_url.query_pairs();
    let list_id = pairs.find(|(k, _)| k == "list");
    if list_id.is_none() {
        message_err!("You need to give a valid playlist link!");
    }
    let list_id = list_id.unwrap();
    let query_params = [
        ("part", "snippet"),
        ("maxResults", "50"),
        ("playlist_id", list_id.1.as_ref()),
        ("key", unsafe { API_KEY.as_ref().unwrap().as_str() }),
    ];
    let url = url::Url::parse_with_params(
        "https://youtube.googleapis.com/youtube/v3/playlistItems",
        &query_params,
    )
    .unwrap();
    let req = ureq::get(url.as_str()).call()?;
    let mut json: PlaylistItems = req.into_json()?;

    let mut out = Vec::with_capacity(json.page_info.total_results as usize);

    out.extend(
        json.items
            .iter()
            .map(|v| format!("https://youtube.com/video?v={}", v.video_id))
            .map(|u| url::Url::parse(u.as_str()).unwrap()),
    );

    while let Some(token) = json.next_page_token {
        let query_params = [
            ("part", "snippet"),
            ("maxResults", "50"),
            ("playlist_id", list_id.1.as_ref()),
            ("pageToken", token.as_str()),
            ("key", unsafe { API_KEY.as_ref().unwrap().as_str() }),
        ];
        let url = url::Url::parse_with_params(
            "https://youtube.googleapis.com/youtube/v3/playlistItems",
            &query_params,
        )
        .unwrap();
        let req = ureq::get(url.as_str()).call()?;
        json = req.into_json()?;

        out.extend(
            json.items
                .iter()
                .map(|v| format!("https://youtube.com/video?q={}", v.video_id))
                .map(|u| url::Url::parse(u.as_str()).unwrap()),
        );
        debug!("out.len() = {}", out.len());
    }

    Ok(out)
}

#[derive(Clone, Debug)]
enum Query {
    Single(String),
    Multiple(Vec<String>),
}

impl SongUrl {
    fn from_url(url: url::Url) -> Self {
        match url.host() {
            Some(url::Host::Domain(u)) => match u {
                "youtube.com" | "youtu.be" | "www.youtube.com" | "www.youtu.be" => match url.path()
                {
                    "/watch" => Self::YoutubeVideo(url),
                    "/playlist" => Self::YoutubePlaylist(url),
                    _ => Self::Query(url.to_string()),
                },
                "spotify.com" => Self::Spotify(url),
                _ => Self::Query(url.to_string()),
            },
            _ => Self::Query(url.to_string()),
        }
    }
    async fn into_query(self) -> Result<Query, CommandError> {
        Ok(match self {
            Self::YoutubeVideo(s) => Query::Single(s.to_string()),
            Self::YoutubePlaylist(p) => Query::Multiple(
                get_yt_playlist_urls(p)?
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>(),
            ),
            Self::Query(s) => Query::Single(s),
            Self::Spotify(_) => {
                todo!()
            }
        })
    }
}

#[group]
#[only_in(guild)]
#[commands(join, play, queue)]
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
        Err(_) => SongUrl::Query(args.remains().unwrap_or("").to_string()),
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
                let song: Input = song.into();
                let metadata = TrackMetadata {
                    url: song.metadata.source_url.clone(),
                    title: song.metadata.title.clone(),
                    duration: song.metadata.duration,
                    added_by: msg.author.id,
                };
                let (track, handle) = songbird::tracks::create_player(song);
                handle
                    .typemap()
                    .write()
                    .await
                    .insert::<TrackMetadataKey>(metadata);
                call_lock.enqueue(track);
            } else if let Ok(y) = songbird::input::restartable::Restartable::ytdl(q, true).await {
                let song: Input = y.into();
                let metadata = TrackMetadata {
                    url: song.metadata.source_url.clone(),
                    title: song.metadata.title.clone(),
                    duration: song.metadata.duration,
                    added_by: msg.author.id,
                };
                if let Some(u) = metadata.url.as_ref() {
                    reply_message!(ctx, msg, format!("Added {url} to the queue", url = u));
                } else {
                    reply_message!(ctx, msg, "Added the song to the queue");
                }
                let (track, handle) = songbird::tracks::create_player(song);
                handle
                    .typemap()
                    .write()
                    .await
                    .insert::<TrackMetadataKey>(metadata);
                call_lock.enqueue(track);
            } else {
                message_err!("❌ No video was found for this song")
            }
        }
        Query::Multiple(list) => {
            for q in list {
                debug!("{}", q);
                let song =
                    songbird::input::restartable::Restartable::ytdl_search(q.as_str(), true).await;
                if let Ok(song) = song {
                    let song: Input = song.into();
                    let metadata = TrackMetadata {
                        url: song.metadata.source_url.clone(),
                        title: song.metadata.title.clone(),
                        duration: song.metadata.duration,
                        added_by: msg.author.id,
                    };
                    let (track, handle) = songbird::tracks::create_player(song);
                    handle
                        .typemap()
                        .write()
                        .await
                        .insert::<TrackMetadataKey>(metadata);
                    call_lock.enqueue(track);
                } else if let Ok(y) = songbird::input::restartable::Restartable::ytdl(q, true).await
                {
                    let song: Input = y.into();
                    let metadata = TrackMetadata {
                        url: song.metadata.source_url.clone(),
                        title: song.metadata.title.clone(),
                        duration: song.metadata.duration,
                        added_by: msg.author.id,
                    };
                    if let Some(u) = metadata.url.as_ref() {
                        reply_message!(ctx, msg, format!("Added {url} to the queue", url = u));
                    } else {
                        reply_message!(ctx, msg, "Added the song to the queue");
                    }
                    let (track, handle) = songbird::tracks::create_player(song);
                    handle
                        .typemap()
                        .write()
                        .await
                        .insert::<TrackMetadataKey>(metadata);
                    call_lock.enqueue(track);
                } else {
                    reply_message!(ctx, msg, "❌ No video was found for this song");
                }
            }
        }
    }
    std::mem::drop(call_lock);
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let page_num = args.parse::<u16>().unwrap_or(0);
    let handler = songbird::get(ctx).await.unwrap();
    let call_mutex = handler.get(msg.guild_id.unwrap());
    let guildname = msg.guild(ctx).await.unwrap().name;
    match call_mutex {
        Some(m) => {
            let lock = m.lock().await;
            let len = (lock.queue().len() as f32 / 10f32).ceil() as u16;
            let page_num = page_num.clamp(0, len);
            let mut pages = Vec::with_capacity(len as usize);
            for (page, song) in lock.queue().current_queue().chunks(10).enumerate() {
                let mut message = CreateMessage::default();
                let mut embed = CreateEmbed::default();
                embed.author(|f| f.name(format!("{}'s queue", guildname.as_str())));
                let mut content = String::new();
                for song in song {
                    let typemap = song.typemap().read().await;
                    let metadata = typemap.get::<TrackMetadataKey>().unwrap();
                    use std::fmt::Write;
                    write!(
                        content,
                        "[{title}]({url})\nAdded by: `{username} [{duration}]`\n",
                        title = metadata.title.clone().unwrap_or_else(|| metadata
                            .url
                            .clone()
                            .unwrap_or_else(|| String::from("Unknown"))),
                        url = metadata.url.as_deref().unwrap_or("https://youtube.com"),
                        username = {
                            let u = msg
                                .guild(ctx)
                                .await
                                .unwrap()
                                .member(ctx, metadata.added_by)
                                .await?;
                            if let Some(nick) = u.nick {
                                format!(
                                    "{nick} ({username}#{disc})",
                                    nick = nick,
                                    username = u.user.name,
                                    disc = u.user.discriminator
                                )
                            } else {
                                format!(
                                    "{username}#{disc}",
                                    username = u.user.name,
                                    disc = u.user.discriminator
                                )
                            }
                        },
                        duration = metadata
                            .duration
                            .map(|d| {
                                chrono::Duration::from_std(d)
                                    .map(|d| {
                                        let secs = d.num_seconds() % 60;
                                        let min = d.num_minutes() % 60;
                                        let hour = d.num_hours();

                                        if hour == 0 {
                                            format!("{m:02}:{s:02}", s = secs, m = min)
                                        } else {
                                            format!(
                                                "{h}:{m:02}:{s:02}",
                                                s = secs,
                                                m = min,
                                                h = hour
                                            )
                                        }
                                    })
                                    .unwrap()
                            })
                            .as_deref()
                            .unwrap_or("Unknown")
                    )?;
                }
                embed.description(content);
                embed.footer(|f| f.text(format!("Page {}/{}", page + 1, len)));
                message.set_embed(embed);
                pages.push(message);

                let menu = serenity_utils::menu::Menu::new(
                    ctx,
                    msg,
                    &pages,
                    MenuOptions {
                        page: page_num as usize,
                        ..Default::default()
                    },
                );

                let _opt_message = menu.run().await?;
            }
        }
        None => {
            message_err!("❌I am not connected to a voice channel.");
        }
    }

    Ok(())
}
