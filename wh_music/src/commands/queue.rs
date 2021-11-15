use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;
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
#[allow(clippy::eval_order_dependence)]
pub async fn queue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let page_num = args.parse::<u16>().unwrap_or(0);
    let handler = songbird::get(ctx).await.unwrap();
    let call_mutex = handler.get(msg.guild_id.unwrap());
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
            let len = (valid_queue.len() as f32 / 10f32).ceil() as u8;
            let page_num = page_num.clamp(0, len as u16);

            let now_playing = match queue.get(0).map(|e| async move {
                let now_playing_info = e.get_info().await;
                if let Err(e) = now_playing_info {
                    return Err(e);
                }
                let now_playing_info = now_playing_info.unwrap();
                let now_playing_typemap = e.typemap().read().await;
                let now_playing_metadata = now_playing_typemap
                    .get::<crate::shared::TrackMetadataKey>()
                    .unwrap();

                Ok(crate::shared::NowPlaying {
                    time_in: now_playing_info.position,
                    song: crate::shared::Song {
                        duration: now_playing_metadata
                            .duration
                            .unwrap_or(std::time::Duration::ZERO),
                        title: {
                            let mut title =
                                now_playing_metadata.title.clone().unwrap_or_else(|| {
                                    now_playing_metadata
                                        .url
                                        .clone()
                                        .unwrap_or_else(|| String::from("Unknown"))
                                });
                            if title.len() > 47 {
                                title = title.chars().take(57).collect::<String>() + "...";
                            }
                            title
                        },
                        added_by: now_playing_metadata.added_by.0,
                        loop_num: match now_playing_info.loops {
                            songbird::tracks::LoopState::Infinite => {
                                crate::shared::LoopState::Infinite
                            }
                            songbird::tracks::LoopState::Finite(1 | 0) => {
                                crate::shared::LoopState::None
                            }
                            songbird::tracks::LoopState::Finite(n) => {
                                crate::shared::LoopState::Finite(n.min(100) as u8)
                            }
                        },
                    },
                })
            }) {
                Some(f) => Some(f.await?),
                None => None,
            };
            let queue =
                match valid_queue
                    .chunks(10)
                    .nth(page_num as usize)
                    .map(|songs| async move {
                        let mut out = arrayvec::ArrayVec::<_, 10>::new();
                        for song in songs.iter() {
                            let song_info = song.get_info().await;
                            if let Err(e) = song_info {
                                return Err(e);
                            }
                            let song_info = song_info.unwrap();

                            let song_typemap = song.typemap().read().await;
                            let song_metadata = song_typemap
                                .get::<crate::shared::TrackMetadataKey>()
                                .unwrap();
                            out.push(crate::shared::Song {
                                duration: song_metadata
                                    .duration
                                    .unwrap_or(std::time::Duration::ZERO),
                                title: {
                                    let mut title =
                                        song_metadata.title.clone().unwrap_or_else(|| {
                                            song_metadata
                                                .url
                                                .clone()
                                                .unwrap_or_else(|| String::from("Unknown"))
                                        });
                                    if title.len() > 47 {
                                        title = title.chars().take(57).collect::<String>() + "...";
                                    }
                                    title
                                },
                                added_by: song_metadata.added_by.0,
                                loop_num: match song_info.loops {
                                    songbird::tracks::LoopState::Infinite => {
                                        crate::shared::LoopState::Infinite
                                    }
                                    songbird::tracks::LoopState::Finite(1 | 0) => {
                                        crate::shared::LoopState::None
                                    }
                                    songbird::tracks::LoopState::Finite(n) => {
                                        crate::shared::LoopState::Finite(n.min(100) as u8)
                                    }
                                },
                            })
                        }
                        Ok(out)
                    }) {
                    Some(f) => Some(f.await?),
                    None => None,
                }
                .unwrap_or_default();
            if let Some(now_playing) = now_playing {
                let typing = msg.channel_id.start_typing(&ctx.http)?;
                let client = reqwest::Client::new();
                let request_builder = client.post(format!(
                    "{base}/api/queue/now_playing",
                    base = *crate::shared::BASE_URL
                ));
                let request_builder = request_builder.json(&now_playing);

                let request = request_builder.send().await?;

                let now_playling_img = request.bytes().await?;
                let queue_list_img;
                typing.stop();
                let mut queue_list_requested = false;
                let _res = msg
                    .channel_id
                    .send_files(
                        &ctx.http,
                        [(&now_playling_img[..], "now_playing.png"), {
                            if !queue.is_empty() {
                                let request_builder = client.post(format!(
                                    "{base}/api/queue/list",
                                    base = *crate::shared::BASE_URL,
                                ));
                                let request_builder =
                                    request_builder.json(&crate::shared::QueueRequest {
                                        page_number: page_num as u8,
                                        total_page_num: len,
                                        queue,
                                        guildid: msg.guild_id.unwrap().0,
                                        callerid: msg.author.id.0,
                                    });
                                let request = request_builder.send().await?;

                                queue_list_img = request.bytes().await?;

                                queue_list_requested = true;
                                (&queue_list_img[..], "rank.png")
                            } else {
                                (&[], "error.png")
                            }
                        }]
                        .into_iter()
                        .take(if queue_list_requested { 2 } else { 1 }),
                        |m| m,
                    )
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
