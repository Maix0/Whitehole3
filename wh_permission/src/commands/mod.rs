add_commands!(Permission, (permission), (permission_manage));

use serenity::framework::standard::{Args, Check, CommandOptions, Reason};
use serenity::model::channel::Message;
use serenity::prelude::Context;

const PERMISSION_MANAGE_CHECK: Check = Check {
    function: check_permission_manage_or_admin,
    name: "permission.manage",
    check_in_help: true,
    display_in_help: true,
};

#[hook]
async fn check_permission_manage_or_admin(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let permission = "permission.manage";
    let res = crate::shared::has_permission(
        ctx,
        msg,
        msg.author.id.0,
        msg.guild_id.unwrap().0,
        permission,
    )
    .await?;
    let discord_permission = msg
        .guild(&ctx.cache)
        .await
        .unwrap()
        .member_permissions(ctx, msg.author.id)
        .await;
    if let Err(e) = &discord_permission {
        return Err(Reason::UserAndLog {
            user: "❌Internal Error".into(),
            log: format!("Internal Error: {}", e),
        });
    }
    let discord_permission = discord_permission.unwrap();
    if !(res || discord_permission.administrator()) {
        return Err(Reason::User(format!(
            "❌You don't have the permission `{}`",
            permission
        )));
    }
    Ok(())
}
