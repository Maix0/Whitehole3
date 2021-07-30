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
        match url.scheme() {
            "spotify" => {
                return Self::Spotify(url);
            }
            _ => {}
        };
        match url.host() {
            Some(url::Host::Domain(u)) => match u {
                "youtube.com" | "youtu.be" | "www.youtube.com" | "www.youtu.be" => match url.path()
                {
                    "/watch" => Self::YoutubeVideo(url),
                    "/playlist" => Self::YoutubePlaylist(url),
                    _ => Self::Query(url.to_string()),
                },
                "spotify.com" | "open.spotify.com" => Self::Spotify(url),
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
            Self::Spotify(q) => SongType::MultipleQuery(handle_spotify(q).await?),
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
#[hook]
async fn handle_spotify(
    uri: url::Url,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    static CLIENT: once_cell::sync::Lazy<aspotify::Client> = once_cell::sync::Lazy::new(|| {
        aspotify::Client::new(
            aspotify::ClientCredentials::from_env_vars("WH_SPOTIFY_ID", "WH_SPOTIFY_SECRET")
                .expect("You need the WH_SPOTIFY_SECRET and WH_SPOTIFY_ID variable set!"),
        )
    });
    let mut out = Vec::new();
    if uri.scheme() == "spotify" {
        let path = uri.path();
        let mut i = path.split(':');
        let (id_type, id) = (i.next(), i.next());
        if id_type.is_none() || id.is_none() {
            message_err!("You provided an invalid spotify uri")
        }
        let id_type = id_type.unwrap();
        let id = id.unwrap();
        match id_type {
            "track" => {
                let track = CLIENT.tracks().get_track(id, None).await;
                if let Err(aspotify::model::Error::Endpoint(e)) = &track {
                    if e.status == 400 {
                        message_err!("Invalid spotify url")
                    }
                }

                out.push(track?.data.name);
            }
            "playlist" => {
                let playlists = CLIENT.playlists();
                let data = playlists.get_playlist(id, None).await;
                if let Err(aspotify::model::Error::Endpoint(e)) = &data {
                    dbg!(&e);
                    if e.status == 404 {
                        message_err!("Invalid spotify url");
                    }
                }
                let page = data?.data.tracks;

                let length = page.total;
                let mut totvec = Vec::with_capacity(length);
                let mut offset = 0;

                while offset < length {
                    let new_page_res = playlists.get_playlists_items(id, 50, offset, None).await;
                    let new_page = new_page_res?.data;
                    let len = new_page.items.len();
                    totvec.extend(new_page.items);
                    offset += len;
                }
                for track in totvec {
                    if let Some(item) = &track.item {
                        let name = match item {
                            aspotify::model::PlaylistItemType::Track(t) => format!(
                                "{} - {}",
                                t.name,
                                t.artists.first().map(|a| a.name.as_str()).unwrap_or("")
                            ),
                            aspotify::model::PlaylistItemType::Episode(e) => {
                                format!("{} - {}", e.name, e.show.name)
                            }
                        };
                        out.push(name);
                    }
                }
            }

            "album" => {
                let albums = CLIENT.albums();
                let data = albums.get_album(id, None).await;
                if let Err(aspotify::model::Error::Endpoint(e)) = &data {
                    if e.status == 400 {
                        message_err!("Invalid spotify url")
                    }
                }

                let page = data?.data.tracks;

                let length = page.total;
                let mut totvec = Vec::with_capacity(length);
                let mut offset = 0;

                while offset < length {
                    let new_page = albums.get_album_tracks(id, 50, offset, None).await?.data;
                    let len = new_page.items.len();
                    totvec.extend(new_page.items);
                    offset += len;
                }
                for track in totvec {
                    let name = format!(
                        "{} - {}",
                        track.name,
                        track.artists.first().map(|a| a.name.as_str()).unwrap_or("")
                    );
                    out.push(name);
                }
            }
            _ => message_err!("unknown spotify uri type"),
        }
    } else {
        let path = uri.path_segments();
        if path.is_none() {
            message_err!("Error when parsing url");
        }
        let mut path = path.unwrap();
        let (type_id, id) = (path.next(), path.next());
        if type_id.is_none() || id.is_none() {
            message_err!("Please input valid spotify url")
        }
        let type_id = type_id.unwrap();
        let id = id.unwrap();

        let uri = url::Url::parse(&format!("spotify:{}:{}", type_id, id)).unwrap();

        out = handle_spotify(uri).await?;
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
#[min_args(1)]
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
        SongType::SingleUrl(q) => {
            play_yt_url(call, q, ctx, msg, true).await?;
        }
        SongType::MultipleUrl(list) => {
            let mut count = 0;
            for q in list {
                play_yt_url(call.clone(), q, ctx, msg, false).await?;
                count += 1;
            }
            reply_message!(ctx, msg, format!("Added {} song(s) to the queue", count));
        }
        SongType::SingleQuery(q) => {
            play_yt_url(call, format!("ytsearch1:{}", q), ctx, msg, true).await?;
        }
        SongType::MultipleQuery(list) => {
            let mut count = 0;
            for q in list {
                play_yt_url(call.clone(), format!("ytsearch1:{}", q), ctx, msg, false).await?;
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
