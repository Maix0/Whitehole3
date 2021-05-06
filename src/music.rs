use std::collections::HashMap;

use serenity::{client::Context, framework::standard::Args, model::id::GuildId};
use serenity::{
    framework::standard::{macros::*, CommandResult},
    prelude::RwLock,
};
use serenity::{model::channel::Message, prelude::TypeMapKey};
use songbird::Call;

pub struct GuildMusicTypeKey;

impl TypeMapKey for GuildMusicTypeKey {
    type Value = GuildMusicManager;
}

#[derive(Default, Debug)]
pub struct GuildMusicManager(HashMap<GuildId, GuildMusic>);

#[derive(Debug)]
pub struct GuildMusic {
    call: std::sync::Arc<std::sync::Mutex<Call>>,
    music_queue: RwLock<Vec<SongUrl>>,
}

#[derive(Clone, Debug)]
pub enum SongUrl {
    Youtube(url::Url),
    Spotify(url::Url),
}

impl SongUrl {
    fn from_url(url: url::Url) -> Option<Self> {
        match url.host() {
            Some(url::Host::Domain(u)) => match u {
                "youtube.com" | "youtu.be" => Some(Self::Youtube(url)),
                "spotify.com" => Some(Self::Spotify(url)),
                _ => None,
            },
            _ => None,
        }
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
            msg.channel(&ctx.cache)
                .await
                .unwrap()
                .guild()
                .unwrap()
                .send_message(&ctx.http, |f| {
                    f.content("You must be in a voice channel to use this command!")
                })
                .await?;
            return Ok(());
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
    let vc = guild.voice_states.get(&ctx.cache.current_user_id().await);
    if vc.is_none() {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let guild_id = guild.id;

        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|x| x.channel_id);

        let connect_to = match channel_id {
            None => {}
            Some(vc) => vc,
        };

        let manager = songbird::get(ctx).await.unwrap();

        let (_handler, res) = manager.join(guild_id, connect_to).await;
        res?;
    }
    args.trimmed().unquoted();
    let url = args.single::<url::Url>();
    if let Err(e) = url {
        error!("{}", e);
        return Ok(());
    }
    let url = url.unwrap();
    let song_url = SongUrl::from_url(url);
    if song_url.is_none() {
        Err("sometihng")?;
    }

    Ok(())
}
