use serenity::framework::standard::{macros::hook, CommandError, CommandResult};
use serenity::model::id::UserId;
use serenity::prelude::TypeMapKey;

pub const MAX_QUEUED_ITEM: usize = 1000;
pub const TIME_BEFORE_LEAVE: u64 = 5 * 60 * 1000;

#[derive(Clone, Debug)]
pub struct TrackMetadataKey;

impl TypeMapKey for TrackMetadataKey {
    type Value = TrackMetadata;
}

#[derive(Clone, Debug)]
pub struct TrackMetadata {
    pub url: Option<String>,
    pub duration: Option<std::time::Duration>,
    pub title: Option<String>,
    pub added_by: UserId,
}

pub struct MusicEventHandler {
    pub(crate) call: std::sync::Arc<tokio::sync::Mutex<songbird::Call>>,
}

#[serenity::async_trait]
impl songbird::events::EventHandler for MusicEventHandler {
    async fn act(&self, _: &songbird::events::EventContext<'_>) -> Option<songbird::events::Event> {
        if self.call.lock().await.queue().is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_millis(TIME_BEFORE_LEAVE)).await;
            if self.call.lock().await.queue().is_empty() {
                match self.call.lock().await.leave().await {
                    Ok(_) => (),
                    Err(e) => error!("Error when disconnecting: {}", e),
                };
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub enum SongType {
    SingleQuery(String),
    MultipleQuery(Vec<String>),
    //
    SingleUrl(url::Url),
    MultipleUrl(Vec<url::Url>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SongUrl {
    YoutubeVideo(url::Url),
    YoutubePlaylist(url::Url),
    Spotify(url::Url),
    Query(String),
    Deezer(url::Url),
}

impl SongUrl {
    pub fn from_url(url: url::Url) -> Self {
        match url.scheme() {
            "spotify" => Self::Spotify(url),
            "http" | "https" => match url.host() {
                Some(url::Host::Domain(u)) => match u {
                    "youtube.com" | "youtu.be" | "www.youtube.com" | "www.youtu.be" => {
                        match url.path() {
                            "/watch" => Self::YoutubeVideo(url),
                            "/playlist" => Self::YoutubePlaylist(url),
                            _ => Self::Query(url.to_string()),
                        }
                    }
                    "spotify.com" | "open.spotify.com" => Self::Spotify(url),
                    "deezer.com" | "www.deezer.com" => Self::Deezer(url),
                    _ => Self::Query(url.to_string()),
                },
                _ => Self::Query(url.to_string()),
            },
            _ => Self::Query(url.to_string()),
        }
    }
    pub async fn into_query(self) -> Result<SongType, CommandError> {
        Ok(match self {
            Self::YoutubeVideo(s) => SongType::SingleUrl(s),
            Self::YoutubePlaylist(p) => SongType::MultipleUrl(get_yt_playlist_urls(p).await?),
            Self::Query(s) => SongType::SingleQuery(s),
            Self::Spotify(q) => SongType::MultipleQuery(handle_spotify(q).await?),
            Self::Deezer(q) => SongType::MultipleQuery(handle_deezer(q).await?),
        })
    }
}
static API_KEY: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    std::env::var("WH_GOOGLE_API_KEY").expect("You need to have `WH_GOOGLE_API_KEY` set")
});

async fn get_yt_playlist_urls<S: AsRef<str>>(query: S) -> CommandResult<Vec<url::Url>> {
    //message_err!("❌Playlist are currently not supported");

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
async fn handle_spotify(uri: url::Url) -> CommandResult<Vec<String>> {
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

async fn handle_deezer(uri: url::Url) -> CommandResult<Vec<String>> {
    // Track URL: https://www.deezer.com/us/track/108733474;

    let mut out = Vec::new();
    let path = uri.path_segments();
    if path.is_none() {
        message_err!("Please input valid deezer url!");
    }

    let path = path.unwrap();
    let mut path = path.skip(1);
    let (typeid, id) = (path.next(), path.next());

    if typeid.is_none() || id.is_none() {
        message_err!("Please input valid deezer url!")
    }
    let (typeid, id) = (typeid.unwrap(), id.unwrap());
    let id: Result<u64, _> = id.parse();

    if let Err(e) = &id {
        both_err!(
            "Please input valid deezer url!",
            format!("Deezer Error: {}", e)
        );
    }

    let id = id.unwrap();

    let deezer_client = deezer::DeezerClient::new();
    match typeid {
        "track" => {
            let res = deezer_client.track(id).await?;

            if res.is_none() {
                message_err!("Couldn't find given track on Deezer!");
            }

            let res = res.unwrap();

            out.push(res.title_short + " - " + &res.artist.name);
        }
        "album" => {
            let res = deezer_client.album(id).await?;

            if res.is_none() {
                message_err!("Couldn't find the given album on Deezer!");
            }

            let res = res.unwrap();

            for track in res.tracks {
                out.push(track.title_short + " - " + &track.artist.name);
            }
        }
        "playlist" => {
            let res = deezer_client.playlist(id).await?;

            if res.is_none() {
                message_err!("Couldn't find the given playlist on Deezer!");
            }

            let res = res.unwrap();

            for track in res.tracks {
                out.push(track.title_short + " - " + &track.artist.name);
            }
        }
        _ => {}
    }

    Ok(out)
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

pub async fn get_video_name(url: &str) -> CommandResult<Option<String>> {
    let client = reqwest::Client::new();

    let q_url = url::Url::parse(url)?;
    let mut pairs = q_url.query_pairs();
    let list_id = pairs.find(|(k, _)| k == "v");
    if list_id.is_none() {
        message_err!("You need to give a valid youtube link link!");
    }
    let list_id = list_id.unwrap();
    let query_params = [
        ("part", "snippet"),
        ("id", list_id.1.as_ref()),
        ("key", API_KEY.as_str()),
    ];
    let url = url::Url::parse_with_params(
        "https://youtube.googleapis.com/youtube/v3/videos",
        &query_params,
    )
    .unwrap();
    let req = client.get(url.as_str()).send().await?;
    let json: serde_json::Value = req.json().await?;
    let pointer = json.pointer("/items/0/snippet/title");
    let title = pointer.map(|v| v.as_str()).flatten().map(|s| s.to_string());
    Ok(title)
}

pub async fn play_yt_url<U>(
    call: std::sync::Arc<tokio::sync::Mutex<songbird::Call>>,
    url: U,
    ctx: &Context,
    msg: &serenity::model::channel::Message,
    show_addition: bool,
) -> CommandResult
where
    U: AsRef<str> + Send + Sync + Clone + 'static,
{
    match songbird::input::restartable::Restartable::ytdl(url, true).await {
        Ok(y) => {
            use songbird::input::Input;
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

/*
   _  ____  _             _ _     _    __
 / / |  _ \| | __ _ _   _| (_)___| |_  \ \
/ /  | |_) | |/ _` | | | | | / __| __|  \ \
\ \  |  __/| | (_| | |_| | | \__ \ |_   / /
 \_\ |_|   |_|\__,_|\__, |_|_|___/\__| /_/
                    |___/
*/

use serenity::prelude::Context;
use wh_database::shared::{DatabaseKey, Id};

#[derive(Clone, Debug)]
struct PlaylistRaw {
    uid: i64,
    userid: i64,
    guildid: i64,
    name: String,
    items: Vec<String>,
}

pub struct Playlist {
    pub uid: i64,
    pub userid: Id,
    pub guildid: Id,
    pub name: String,
    pub items: Vec<String>,
}

impl PlaylistRaw {
    fn into_processed(self) -> Playlist {
        Playlist {
            uid: self.uid,
            userid: self.userid.into(),
            guildid: self.guildid.into(),
            name: self.name,
            items: self.items,
        }
    }
}
pub async fn get_all_playlist(ctx: &Context, guildid: u64) -> CommandResult<Vec<Playlist>> {
    use serenity::futures::stream::StreamExt;
    let lock = ctx.data.read().await;
    let db = lock.get::<DatabaseKey>().unwrap();
    let res = query_as!(
        PlaylistRaw,
        "SELECT * FROM user_playlist WHERE guildid = $1::int8",
        Id(guildid) as _,
    )
    .fetch(db);

    let iter = res
        .map(|v| v.map(|v| v.into_processed()))
        .collect::<Vec<_>>()
        .await;
    let mut out = Vec::with_capacity(iter.len());
    for res in iter {
        out.push(res?);
    }
    Ok(out)
}

pub async fn get_playlist(
    ctx: &Context,
    guildid: u64,
    name: &str,
) -> CommandResult<Option<Playlist>> {
    let lock = ctx.data.read().await;
    let db = lock.get::<DatabaseKey>().unwrap();
    query_as!(
        PlaylistRaw,
        "SELECT * FROM user_playlist WHERE guildid = $1::int8 AND name = UPPER($2::varchar(32))",
        Id(guildid) as _,
        name
    )
    .fetch_optional(db)
    .await
    .map(|res| res.map(|r| r.into_processed()))
    .map_err(|e| Box::new(e).into())
}

pub async fn create_playlist_if_not_exist(
    ctx: &Context,
    name: &str,
    user_id: u64,
    guildid: u64,
) -> CommandResult<bool> {
    let existing = get_playlist(ctx, guildid, name).await?;
    if existing.is_some() {
        return Ok(false);
    }
    if name.len() > 32 {
        message_err!("Playlist name too long (32 characters maximum)");
    }
    let lock = ctx.data.read().await;
    let db = lock.get::<DatabaseKey>().unwrap();

    query!("INSERT INTO user_playlist (userid, guildid, name, items) VALUES ($1::int8, $2::int8, $3::varchar(32),  $4::text[])", 
        Id(user_id) as _,
        Id(guildid) as _,
        name,
        &[][..]
    ).execute(db).await?;

    Ok(true)
}
