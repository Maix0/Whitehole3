use serenity::framework::standard::{macros::*, CommandResult};
use serenity::{
    builder::{CreateEmbed, CreateMessage},
    model::channel::Message,
};
use serenity::{client::Context, framework::standard::Args};

#[command]
#[only_in(guilds)]
#[num_args(0_1)]
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
            std::mem::drop(lock);
            let len = (queue.len() as f32 / 10f32).ceil() as u16;
            let page_num = page_num.clamp(0, len);
            if let Some((page, song)) = queue.chunks(10).enumerate().nth(page_num as usize) {
                let mut message = CreateMessage::default();
                let mut embed = CreateEmbed::default();
                embed.author(|f| f.name(format!("{}'s queue", guildname.as_str())));
                let mut content = String::new();

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
                msg.channel_id
                    .send_message(&ctx.http, |c| {
                        *c = message;
                        c
                    })
                    .await?;
            } else {
                reply_message!(ctx, msg, "The queue is empty");
            }
        }
        None => {
            reply_message!(ctx, msg, "❌I am not connected to a voice channel.");
        }
    }

    Ok(())
}
