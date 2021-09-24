use serenity::framework::standard::{macros::*, CommandResult};
use serenity::{
    builder::{CreateEmbed, CreateMessage},
    model::channel::Message,
};
use serenity::{client::Context, framework::standard::Args};

#[command]
#[only_in(guilds)]
#[aliases("q")]
#[min_args(0)]
#[max_args(1)]
#[usage("[page?]")]
#[example("")]
/// Get the bot's queue
/// If a page number is appended to the command, it will show the page asked otherwise it will show the first page
pub async fn queue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let page_num = args.parse::<u16>().unwrap_or(0);
    let handler = songbird::get(ctx).await.unwrap();
    let call_mutex = handler.get(msg.guild_id.unwrap());
    let guildname = msg.guild(ctx).await.unwrap().name;
    match call_mutex {
        Some(m) => {
            let lock = m.lock().await;
            let queue = lock.queue().current_queue();
            let valid_queue;
            if queue.len() > 1 {
                valid_queue = &queue[1..];
            } else {
                valid_queue = &[];
            }
            std::mem::drop(lock);
            let len = (valid_queue.len() as f32 / 10f32).ceil() as u16;
            let page_num = page_num.clamp(0, len);
            if let Some((page, song)) = valid_queue.chunks(10).enumerate().nth(page_num as usize) {
                let mut message = CreateMessage::default();
                let mut embed = CreateEmbed::default();
                embed.author(|f| f.name(format!("{}'s queue", guildname.as_str())));
                let mut content = String::new();
                let first_typemap = queue[0].typemap().read().await;
                let first_metadata = first_typemap
                    .get::<crate::shared::TrackMetadataKey>()
                    .unwrap();
                let current_playing = format!(
                    "[{title}]({url}) \nAdded by `{username} [{duration}]`",
                    title = {
                        let mut title = first_metadata.title.clone().unwrap_or_else(|| {
                            first_metadata
                                .url
                                .clone()
                                .unwrap_or_else(|| String::from("Unknown"))
                        });
                        if title.len() > 37 {
                            title = title.chars().take(37).collect::<String>() + "...";
                        }
                        title
                    },
                    url = first_metadata
                        .url
                        .as_deref()
                        .unwrap_or("https://youtube.com"),
                    username = {
                        let u = msg
                            .guild(ctx)
                            .await
                            .unwrap()
                            .member(ctx, first_metadata.added_by)
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
                    duration = first_metadata
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
                                        format!("{h}:{m:02}:{s:02}", s = secs, m = min, h = hour)
                                    }
                                })
                                .unwrap()
                        })
                        .as_deref()
                        .unwrap_or("Unknown"),
                );

                for (index, song) in song.iter().enumerate() {
                    let typemap = song.typemap().read().await;
                    let metadata = typemap.get::<crate::shared::TrackMetadataKey>().unwrap();
                    use std::fmt::Write;
                    write!(
                        content,
                        "`{index})`[{title}]({url})\n\tAdded by: `{username} [{duration}]`\n",
                        title = {
                            let mut title = metadata.title.clone().unwrap_or_else(|| {
                                metadata
                                    .url
                                    .clone()
                                    .unwrap_or_else(|| String::from("Unknown"))
                            });
                            if title.len() > 47 {
                                title = title.chars().take(57).collect::<String>() + "...";
                            }
                            title
                        },
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
                            .unwrap_or("Unknown"),
                        index = page_num as usize * 10 + index as usize + 1
                    )?;
                }
                embed.field("Now Playing", current_playing, false);
                embed.field("Queue", content, false);
                embed.footer(|f| f.text(format!("Page {}/{}", page + 1, len)));
                message.set_embed(embed);
                msg.channel_id
                    .send_message(&ctx.http, |c| {
                        *c = message;
                        c
                    })
                    .await?;
            } else if let Some(i) = queue.get(0) {
                let first_typemap = i.typemap().read().await;
                let first_metadata = first_typemap
                    .get::<crate::shared::TrackMetadataKey>()
                    .unwrap();
                let current_playing = format!(
                    "[{title}]({url}) \nAdded by `{username} [{duration}]`",
                    title = {
                        let mut title = first_metadata.title.clone().unwrap_or_else(|| {
                            first_metadata
                                .url
                                .clone()
                                .unwrap_or_else(|| String::from("Unknown"))
                        });
                        if title.len() > 57 {
                            title = title.chars().take(57).collect::<String>() + "...";
                        }
                        title
                    },
                    url = first_metadata
                        .url
                        .as_deref()
                        .unwrap_or("https://youtube.com"),
                    username = {
                        let u = msg
                            .guild(ctx)
                            .await
                            .unwrap()
                            .member(ctx, first_metadata.added_by)
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
                    duration = first_metadata
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
                                        format!("{h}:{m:02}:{s:02}", s = secs, m = min, h = hour)
                                    }
                                })
                                .unwrap()
                        })
                        .as_deref()
                        .unwrap_or("Unknown"),
                );

                let mut message = CreateMessage::default();
                let mut embed = CreateEmbed::default();
                embed.field("Now Playing", current_playing, false);
                embed.author(|f| f.name(format!("{}'s queue", guildname.as_str())));
                message.set_embed(embed);
                msg.channel_id
                    .send_message(&ctx.http, |c| {
                        *c = message;
                        c
                    })
                    .await?;
            } else {
                reply_message!(ctx, msg, fluent!(MUSIC_empty_queue));
            }
        }
        None => {
            reply_message!(ctx, msg, fluent!(MUSIC_voice_not_connected));
        }
    }

    Ok(())
}
