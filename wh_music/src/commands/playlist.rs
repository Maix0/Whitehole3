use playlist::*;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::Context;
#[command]
#[only_in(guilds)]
#[sub_commands(add, remove, new, delete, view, list)]
async fn playlist(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

mod playlist {
    use serenity::framework::standard::{macros::*, Args, CommandResult};
    use serenity::model::channel::Message;
    use serenity::prelude::Context;

    #[command]
    #[only_in(guilds)]
    async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let name = args.single_quoted();
        if name.is_err() {
            message_err!("You need to provide a valid playlist name!")
        }
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[example("metal")]
    #[usage("[playlist name]")]
    async fn new(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let name = args.single_quoted::<String>();
        if name.is_err() {
            message_err!("You need to provide a name for a new playlist!\nPut it within quotes if it contains spaces");
        }
        let name = name.unwrap();
        if name.len() > 32 {
            message_err!("Playlist name is restricted to no more than 32 characters");
        }
        let created = crate::shared::create_playlist_if_not_exist(
            ctx,
            &name,
            msg.author.id.0,
            msg.guild_id.unwrap().0,
        )
        .await?;
        if created {
            reply_message!(ctx, msg, "The playlist has been created");
        } else {
            reply_message!(ctx, msg, "A playlist with that name already exists!");
        }
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    async fn delete(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let name = args.single_quoted::<String>();
        if name.is_err() {
            message_err!("You need to provide a playlist name!");
        }
        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
        let name = name.unwrap();
        let res = query!("
WITH deleted AS 
    (DELETE FROM user_playlist WHERE userid = $1::int8 AND guildid = $2::int8 AND name = $3::varchar(32) RETURNING *) 
SELECT count(*) FROM deleted", wh_database::shared::Id(msg.author.id.0)as _, wh_database::shared::Id(msg.guild_id.unwrap().0) as _, name ).fetch_one(db).await?;
        if res.count == Some(0) {
            message_err!(
                "Couldn't remove the playlists, maybe you misspeled it or you aren't the owner"
            )
        } else {
            reply_message!(ctx, msg, "The playlist has been deleted");
        }
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    async fn view(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        use serenity::builder::{CreateEmbed, CreateMessage};
        let name = args.single_quoted::<String>();
        let page_num = args.single::<u16>().unwrap_or(0);
        if name.is_err() {
            message_err!("You need to provide a playlist name");
        }
        let name = name.unwrap();
        let playlist = crate::shared::get_playlist(ctx, msg.guild_id.unwrap().0, &name).await?;
        if playlist.is_none() {
            message_err!("This playlist doesn't exist");
        }
        let playlist = playlist.unwrap();

        let len = (playlist.items.len() as f32 / 10f32).ceil() as u16;
        let page_num = page_num.clamp(0, len);
        if let Some((page, song)) = playlist.items.chunks(10).enumerate().nth(page_num as usize) {
            let mut message = CreateMessage::default();
            let mut embed = CreateEmbed::default();
            embed.author(|f| f.name(format!("Playlist - {}", name)));
            let mut content = String::new();

            for song in song {
                use std::fmt::Write;
                write!(
                    content,
                    "[{title}]({url})\n",
                    title = crate::shared::get_video_name(&song)
                        .await?
                        .as_deref()
                        .unwrap_or("Unknown"),
                    url = &song,
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
    async fn list(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let page_num = args.single::<u16>().unwrap_or(0);
        let playlists = crate::shared::get_all_playlist(ctx, msg.guild_id.unwrap().0).await?;
        if playlists.is_empty() {
            message_err!("This server doesn't have any playlists");
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
                write!(
                    content,
                    "`{name}` *created by* **{user}**\n",
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
}
