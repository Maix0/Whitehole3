use serenity::client::Context;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::channel::Message;

#[command("loop")]
#[only_in(guilds)]
#[usage("loop <?num>")]
pub async fn loop_cmd(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let manager = songbird::get(&ctx).await.unwrap();

    let call = manager.get(msg.guild_id);

    if call.is_none() {
        message_err!(fluent!(MUSIC_voice_not_connected));
    }
    let call = call.unwrap();

    let loop_state = {
        let track = call.lock().await.queue().current();
        if track.is_none() {
            message_err!(fluent!(MUSIC_empty_queue));
        }
        let track = track.unwrap();

        let metadata = track.get_info().await.unwrap();
        match metadata.loops {
            songbird::tracks::LoopState::Finite(0) => None,
            n => Some(n),
        }
    };
    let mut new_loop_state = songbird::tracks::LoopState::Finite(0);
    let num_arg: Option<usize> = args.single().ok();

    if loop_state.is_none() {
        match num_arg {
            Some(n) => new_loop_state = songbird::tracks::LoopState::Finite(n),
            None => new_loop_state = songbird::tracks::LoopState::Infinte,
        }
    } else {
        match num_arg {
            Some(n) => new_loop_state = songbird::tracks::LoopState::Finite(n),
            None => new_loop_state = songbird::tracks::LoopState::Finite(0),
        }
    }

    match new_loop_state {
        songbird::tracks::LoopState::Finite(n) => {
            if n = 0 {
                call.lock().await.queue().current().unwrap().disable_loop();
            } else {
                call.lock().await.queue().current().unwrap().loop_for(n);
            }
        }
        songbird::tracks::LoopState::Infinite => {
            call.lock().await.queue().current().unwrap().enable_loop()
        }
    }

    Ok(())
}
