use serenity::framework::standard::{macros::*, CommandResult};
use serenity::{
    builder::{CreateEmbed, CreateMessage},
    model::channel::Message,
};
use serenity::{client::Context, framework::standard::Args};
use serenity_utils::prelude::MenuOptions;

#[command]
#[only_in(guilds)]
pub async fn queue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let page_num = args.parse::<u16>().unwrap_or(0);
    let handler = songbird::get(ctx).await.unwrap();
    let call_mutex = handler.get(msg.guild_id.unwrap());
    let guildname = msg.guild(ctx).await.unwrap().name;
    match call_mutex {
        Some(m) => {
            let lock = m.lock().await;
            let queue = lock.queue().current_queue();
            let len = (queue.len() as f32 / 10f32).ceil() as u16;
            let page_num = page_num.clamp(0, len);
            let mut pages = Vec::with_capacity(len as usize);
            for (page, song) in queue.chunks(10).enumerate() {
                let mut message = CreateMessage::default();
                let mut embed = CreateEmbed::default();
                embed.author(|f| f.name(format!("{}'s queue", guildname.as_str())));
                let mut content = String::new();
                debug!("page: {}, song: {}", page, song.len());
                for song in song {
                    let typemap = song.typemap().read().await;
                    let metadata = typemap.get::<crate::shared::TrackMetadataKey>().unwrap();
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
            }

            let menu = serenity_utils::menu::Menu::new(
                ctx,
                msg,
                &pages,
                MenuOptions {
                    page: page_num as usize,
                    controls: vec![
                        serenity_utils::menu::Control::new(
                            serenity::model::channel::ReactionType::Unicode("◀️".into()),
                            std::sync::Arc::new(|m, r| {
                                Box::pin(serenity_utils::menu::prev_page(m, r))
                            }),
                        ),
                        serenity_utils::menu::Control::new(
                            serenity::model::channel::ReactionType::Unicode("▶️".into()),
                            std::sync::Arc::new(|m, r| {
                                Box::pin(serenity_utils::menu::next_page(m, r))
                            }),
                        ),
                    ],
                    ..Default::default()
                },
            );

            let _opt_message = menu.run().await?;
        }
        None => {
            reply_message!(ctx, msg, "❌I am not connected to a voice channel.");
        }
    }

    Ok(())
}
