add_commands!(
    Music,
    (clear, join, pause, play, queue, resume, skip),
    (none)
);

add_commands!(MusicPriv, (move_cmd, remove, leave), (music_manage));

use serenity::framework::standard::macros::check;
use serenity::framework::standard::Reason;
use serenity::model::channel::Message;
use serenity::prelude::Context;

#[check]
#[name("music_manage")]
async fn music_privileged(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    let permisison = "music.manage";
    let has = wh_permission::shared::has_permission(
        ctx,
        msg.author.id.0,
        msg.guild_id.unwrap().0,
        permission,
    )
    .await?;
    if has {
        return Ok(());
    } else {
        return Err(Reason::User(format!(
            "You don't have the permission `{}` required to use this command",
            permission
        )));
    }
}

#[check]
#[name("none")]
async fn empty_check(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    Ok(())
}
