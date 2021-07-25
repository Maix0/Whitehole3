use serenity::framework::standard::{macros::*, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::{client::Context, framework::standard::Args};
use songbird::input::Input;

#[derive(Clone, Debug)]
enum SongType {
    SingleQuery(String),
    MultipleQuery(Vec<String>),
    //
    SingleUrl(url::Url),
    MultipleUrl(Vec<url::Url>),
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
    async fn into_query(self) -> Result<SongType, CommandError> {
        Ok(match self {
            Self::YoutubeVideo(s) => SongType::SingleUrl(s),
            Self::YoutubePlaylist(p) => SongType::MultipleUrl(get_yt_playlist_urls(p).await?),
            Self::Query(s) => SongType::SingleQuery(s),
            Self::Spotify(_) => {
                todo!()
            }
        })
    }
}

async fn get_yt_playlist_urls<S: AsRef<str>>(query: S) -> CommandResult<Vec<url::Url>> {
    //message_err!("❌Playlist are currently not supported");
    static API_KEY: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
        std::env::var("WH_GOOGLE_API_KEY").expect("You need to have `WH_GOOGLE_API_KEY` set")
    });

    let client = reqwest::Client::new();

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
        ("key", API_KEY.as_str()),
    ];
    let url = url::Url::parse_with_params(
        "https://youtube.googleapis.com/youtube/v3/playlistItems",
        &query_params,
    )
    .unwrap();
    let req = client.get(url.as_str()).send().await?;
    let mut json: PlaylistItems = req.json().await?;

    let mut out = Vec::with_capacity(json.page_info.total_results as usize);

    out.extend(
        json.items
            .iter()
            .map(|v| {
                format!(
                    "https://youtube.com/watch?v={}",
                    v.snippet.resource_id.video_id
                )
            })
            .map(|u| url::Url::parse(u.as_str()).unwrap()),
    );

    while let Some(token) = json.next_page_token {
        let query_params = [
            ("part", "snippet"),
            ("maxResults", "50"),
            ("playlist_id", list_id.1.as_ref()),
            ("pageToken", token.as_str()),
            ("key", API_KEY.as_str()),
        ];
        let url = url::Url::parse_with_params(
            "https://youtube.googleapis.com/youtube/v3/playlistItems",
            &query_params,
        )
        .unwrap();
        let req = client.get(url.as_str()).send().await?;
        json = req.json().await?;

        out.extend(
            json.items
                .iter()
                .map(|v| {
                    format!(
                        "https://youtube.com/video?v={}",
                        v.snippet.resource_id.video_id
                    )
                })
                .map(|u| url::Url::parse(u.as_str()).unwrap()),
        );
    }
    Ok(out)
}

#[derive(Clone, Debug)]
pub enum SongUrl {
    YoutubeVideo(url::Url),
    YoutubePlaylist(url::Url),
    Spotify(url::Url),
    Query(String),
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
    #[serde(alias = "snippet")]
    snippet: Snippet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Snippet {
    #[serde(alias = "resourceId")]
    resource_id: ResourceId,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ResourceId {
    #[serde(alias = "videoId")]
    video_id: String,
}

#[command]
#[only_in(guilds)]
#[usage("[query or url]")]
#[example("https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
#[num_args(1)]
/// Make the bot play the music
/// the query can be a youtube video, a youtube playlist, a simple query or a spotify song/playlist url
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

    match song_query {
        SongType::SingleUrl(q) => {
            if let Ok(y) = songbird::input::restartable::Restartable::ytdl(q, true).await {
                let song: Input = y.into();
                let metadata = crate::shared::TrackMetadata {
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
                    .insert::<crate::shared::TrackMetadataKey>(metadata);
                let mut call_lock = call.lock().await;
                call_lock.enqueue(track);
            } else {
                message_err!("❌ No video was found for this song");
            }
        }
        SongType::MultipleUrl(list) => {
            //message_err!("❌ Queueing Multiple songs at once is disable for now!");
            let mut count = 0;
            for q in list {
                if let Ok(y) = songbird::input::restartable::Restartable::ytdl(q, true).await {
                    let song: Input = y.into();
                    let metadata = crate::shared::TrackMetadata {
                        url: song.metadata.source_url.clone(),
                        title: song.metadata.title.clone(),
                        duration: song.metadata.duration,
                        added_by: msg.author.id,
                    };
                    count += 1;
                    let (track, handle) = songbird::tracks::create_player(song);
                    handle
                        .typemap()
                        .write()
                        .await
                        .insert::<crate::shared::TrackMetadataKey>(metadata);

                    let mut call_lock = call.lock().await;
                    call_lock.enqueue(track);
                    std::mem::drop(call_lock)
                } else {
                    reply_message!(ctx, msg, "❌ No video was found for this song");
                }
            }
            reply_message!(ctx, msg, format!("Added {} song(s) to the queue", count));
        }
        _ => warn!("Todo SongType::*Query"),
    }
    Ok(())
}
