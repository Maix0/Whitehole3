add_commands!(Music, (clear, join, pause, play, queue, resume, skip), ());

add_commands!(MusicPriv, (move_cmd, remove, leave), (music_manage));

use serenity::framework::standard::{Check, Reason};
use serenity::model::channel::Message;
use serenity::prelude::Context;

const MUSIC_MANAGE_CHECK: Check = Check {
    function: |a, b, _, _| music_privileged(a, b),
    name: "music.manage",
    display_in_help: true,
    check_in_help: true,
};

#[hook]
async fn music_privileged(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    let permissison = "music.manage";
    let has = wh_permission::shared::has_permission(
        ctx,
        msg.author.id.0,
        msg.guild_id.unwrap().0,
        permissison,
    )
    .await?;
    if has {
        return Ok(());
    } else {
        return Err(Reason::User(format!(
            "You don't have the permission `{}` required to use this command",
            permissison
        )));
    }
}
