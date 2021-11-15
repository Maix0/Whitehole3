use playlist_cmd::*;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::Context;
#[command]
#[only_in(guilds)]
#[sub_commands(add, remove, new, delete, view, list, play)]
/// This command does nothing on its own, must use the subcommands add, remove, view, list, delete
async fn playlist(_: &Context, _: &Message) -> CommandResult {
    Ok(())
}

mod playlist_cmd {
    use serenity::framework::standard::{macros::*, Args, CommandResult};
    use serenity::model::channel::Message;
    use serenity::prelude::Context;

    #[command]
    #[only_in(guilds)]
    #[usage("[name] [query or url]")]
    #[example("\"meme music\" never gonna give you up")]
    #[min_args(2)]
    /// Add a music to a playlist
    async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let name = args.single_quoted::<String>();
        if name.is_err() {
            message_err!(fluent!(MUSIC_ARG_playlist_name))
        }
        let name = name.unwrap();
        let url = url::Url::parse(args.remains().unwrap_or(""));
        let song_url = match url {
            Ok(url) => crate::shared::SongUrl::from_url(url),
            Err(_) => crate::shared::SongUrl::Query(args.remains().unwrap_or("").to_string()),
        };
        if crate::shared::SongUrl::Query("".to_string()) == song_url {
            message_err!(fluent!(MUSIC_ARG_query_or_url));
        }

        let song_query = song_url.into_query().await?;

        let mut urls = Vec::<String>::new();
        match song_query {
            crate::shared::SongType::SingleUrl(q) => {
                urls.push(q.to_string());
            }
            crate::shared::SongType::MultipleUrl(list) => {
                urls.extend(list.iter().map(ToString::to_string));
            }
            crate::shared::SongType::SingleQuery(q) => {
                use std::process::{Command, Stdio};
                let mut command = Command::new("youtube-dl");
                command
                    .arg("--get-id")
                    .arg(format!("ytsearch:{}", q))
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped());
                let output = command.output()?;
                if output.stdout.is_empty() {
                    message_err!(format!(fluent!(MUSIC_not_found_video), q));
                }
                let id = String::from_utf8(output.stdout).unwrap();
                urls.push(format!(" https://www.youtube.com/watch?v={}", id));
            }
            crate::shared::SongType::MultipleQuery(list) => {
                use std::process::{Command, Stdio};

                for q in &list {
                    let mut command = Command::new("youtube-dl");
                    command
                        .arg("--get-id")
                        .arg(format!("ytsearch:{}", q))
                        .stdin(Stdio::null())
                        .stdout(Stdio::piped());
                    let output = command.output()?;
                    if output.stdout.is_empty() {
                        message_err!(format!(fluent!(MUSIC_not_found_video), q));
                    }
                    let id = String::from_utf8(output.stdout).unwrap();
                    urls.push(format!("https://www.youtube.com/watch?v={}", id));
                }
            }
        }
        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

        query!("UPDATE user_playlist SET items = array_distinct(array_cat(items, $3::text[])) WHERE name = UPPER($1::varchar(32)) AND guildid = $2::int8;", 
        name, wh_database::shared::Id(msg.guild_id.unwrap().0) as _, &urls).execute(db).await?;
        reply_message!(
            ctx,
            msg,
            format!(
                fluent!(MUSIC_songs_added_playlist),
                urls.len(),
                if urls.len() <= 1 { "" } else { "s" },
                name
            )
        );

        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[usage("[name] [index]")]
    #[example("memes 1")]
    #[num_args(2)]
    /// Remove the music at the given index for the given playlist
    async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let name = args.single_quoted::<String>();
        if name.is_err() {
            message_err!(fluent!(MUSIC_ARG_playlist_name));
        }
        let name = name.unwrap();
        let index = args.single::<u16>();
        if index.is_err() {
            message_err!(fluent!(MUSIC_ARG_invalid_number));
        }
        let index = index.unwrap();
        if index == 0 {
            message_err!(fluent!(MUSIC_ARG_invalid_number));
        }

        let playlist = crate::shared::get_playlist(ctx, msg.guild_id.unwrap().0, &name).await?;
        if playlist.is_none() {
            message_err!(fluent!(MUSIC_playlist_not_exist));
        }
        let playlist = playlist.unwrap();
        if playlist.userid.0 != msg.author.id.0 {
            message_err!(fluent!(MUSIC_playlist_not_owner));
        }

        if playlist.items.get(index as usize - 1).is_none() {
            message_err!(fluent!(MUSIC_ARG_invalid_number));
        }

        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

        let res = query!("UPDATE user_playlist SET items = array_distinct(array_diff(items, $3::text[])) WHERE guildid = $1::int8 AND name = UPPER($2::varchar(32))",
            wh_database::shared::Id(msg.guild_id.unwrap().0) as _, name, &[playlist.items[index as usize - 1].as_str()][..] as _
        ).execute(db).await?;

        if res.rows_affected() < 1 {
            message_err!(fluent!(MUSIC_playlist_failed_remove));
        }
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[example("memes")]
    #[usage("[name]")]
    #[num_args(1)]
    /// Create a new playlist. The name must be 32 character long or shorter and if it include space wrap it inside double quotes
    async fn new(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let name = args.single_quoted::<String>();
        if name.is_err() {
            message_err!(fluent!(MUSIC_ARG_playlist_name));
        }
        let name = name.unwrap();
        if name.len() > 32 {
            message_err!(fluent!(MUSIC_ARG_playlist_name_too_long));
        }
        let created = crate::shared::create_playlist_if_not_exist(
            ctx,
            &name,
            msg.author.id.0,
            msg.guild_id.unwrap().0,
        )
        .await?;
        if created {
            reply_message!(ctx, msg, fluent!(MUSIC_playlist_created));
        } else {
            reply_message!(ctx, msg, fluent!(MUSIC_playlist_exists));
        }
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[num_args(1)]
    #[example("memes")]
    #[usage("[name]")]
    /// Delete the playlist. You must be the owner of said playlist
    async fn delete(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let name = args.single_quoted::<String>();
        if name.is_err() {
            message_err!(fluent!(MUSIC_ARG_playlist_name));
        }
        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
        let name = name.unwrap();
        let res = query!("
WITH deleted AS 
    (DELETE FROM user_playlist WHERE userid = $1::int8 AND guildid = $2::int8 AND name = UPPER($3::varchar(32)) RETURNING *) 
SELECT count(*) FROM deleted", wh_database::shared::Id(msg.author.id.0)as _, wh_database::shared::Id(msg.guild_id.unwrap().0) as _, name ).fetch_one(db).await?;
        if res.count == Some(0) {
            message_err!(fluent!(MUSIC_playlist_failed_delete))
        } else {
            reply_message!(ctx, msg, fluent!(MUSIC_playlist_deleted));
        }
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[min_args(1)]
    #[max_args(2)]
    #[usage("[name] [?page]")]
    #[example("memes 2")]
    /// View the playlist items at the given page (default to page 1 if not specified)
    async fn view(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        use serenity::builder::{CreateEmbed, CreateMessage};
        let name = args.single_quoted::<String>();
        let page_num = args.single::<u16>().unwrap_or(0);
        if name.is_err() {
            message_err!(fluent!(MUSIC_ARG_playlist_name));
        }
        let name = name.unwrap();
        let playlist = crate::shared::get_playlist(ctx, msg.guild_id.unwrap().0, &name).await?;
        if playlist.is_none() {
            message_err!(fluent!(MUSIC_playlist_not_exist));
        }
        let playlist = playlist.unwrap();

        let len = (playlist.items.len() as f32 / 10f32).ceil() as u16;
        let page_num = page_num.clamp(0, len);
        if let Some((page, song)) = playlist.items.chunks(10).enumerate().nth(page_num as usize) {
            let mut message = CreateMessage::default();
            let mut embed = CreateEmbed::default();
            embed.author(|f| f.name(format!("Playlist - {}", name)));
            let mut content = String::new();

            for (index, song) in song.iter().enumerate() {
                let mut text = crate::shared::get_video_name(song)
                    .await?
                    .unwrap_or_else(|| "Unknown".to_string());
                if text.len() > 57 {
                    text = text.chars().take(57).collect::<String>() + "...";
                }
                use std::fmt::Write;
                writeln!(
                    content,
                    "`{index})` [{title}]({url})",
                    title = text,
                    url = &song,
                    index = page_num * 10 + index as u16 + 1
                )?;
            }
            embed.description(content);
            embed.footer(|f| f.text(format!("Page {}/{}", page + 1, len)));
            message.set_embed(embed);
            msg.channel_id
                .send_message(&ctx.http, |c| {
                    *c = message;
                    c
                })
                .await?;
        }
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[max_args(1)]
    #[usage("[?page]")]
    #[example("2")]
    /// View the all the guild's playlist
    async fn list(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let page_num = args.single::<u16>().unwrap_or(0);
        let playlists = crate::shared::get_all_playlist(ctx, msg.guild_id.unwrap().0).await?;
        if playlists.is_empty() {
            message_err!(fluent!(MUSIC_no_playlists));
        }

        let len = (playlists.len() as f32 / 10f32).ceil() as u16;
        let page_num = page_num.clamp(0, len);
        if let Some((page, playlist)) = playlists.chunks(10).enumerate().nth(page_num as usize) {
            use serenity::builder::*;
            let mut message = CreateMessage::default();
            let mut embed = CreateEmbed::default();
            let guild_name = msg.guild_id.unwrap().to_partial_guild(ctx).await?.name;
            embed.author(|f| f.name(format!("Playlists - {}", guild_name)));

            let mut content = String::new();
            for p in playlist {
                use std::fmt::Write;
                writeln!(
                    content,
                    "`{name:^32}` *created by* **{user}**",
                    name = &p.name,
                    user = &serenity::model::id::UserId(p.userid.0)
                        .to_user(ctx)
                        .await?
                        .tag(),
                )?;
            }
            embed.description(content);
            embed.footer(|f| f.text(format!("Page {}/{}", page + 1, len)));
            message.set_embed(embed);
            msg.channel_id
                .send_message(&ctx.http, |c| {
                    *c = message;
                    c
                })
                .await?;
        }
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[max_args(2)]
    #[min_args(1)]
    #[usage("[name] [shuffle]")]
    #[example("memes true")]
    /// Add the playlist to the queue
    /// The last argument is specifying if the playlist is shuffle before adding it to the queue (true => shuffled; false => not shuffled)
    pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let name = args.single_quoted::<String>();
        if name.is_err() {
            message_err!(fluent!(MUSIC_ARG_playlist_name));
        }
        let name = name.unwrap();

        let playlist = crate::shared::get_playlist(ctx, msg.guild_id.unwrap().0, &name).await?;
        if playlist.is_none() {
            message_err!(fluent!(MUSIC_playlist_not_exist));
        }
        let playlist = playlist.unwrap();

        let random = args.single::<bool>().unwrap_or(false);

        let guild = msg.guild(&ctx.cache).await.unwrap();
        let vc = guild.voice_states.get(&ctx.cache.current_user_id().await);
        if vc.is_none() {
            let guild_id = guild.id;

            let channel_id = guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|x| x.channel_id);

            let connect_to = match channel_id {
                None => {
                    message_err!(fluent!(MUSIC_voice_not_connected))
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
        let call = manager.get(msg.guild_id.unwrap()).unwrap();

        let mut songs = playlist.items;
        if random {
            use rand::seq::SliceRandom;
            songs.shuffle(&mut rand::thread_rng());
        }
        for song in songs {
            crate::shared::play_yt_url(call.clone(), song, ctx, msg, false).await?;
        }

        Ok(())
    }
}
