extern crate serenity;
extern crate songbird;
#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate wh_core;

use serenity::framework::standard::{macros::*, CommandResult};
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
    Youtube(url::Url),
    Spotify(url::Url),
    Query(String),
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

    Ok(())
}
