use serenity::client::Context;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[usage("[@user]")]
#[example("@-|Maix|")]
#[max_args(1)]
/// Show the rank card of the user that called this command or the user mentioned
pub async fn rank(ctx: &Context, msg: &Message) -> CommandResult {
    let usr = msg.mentions.first().map(|u| u.id).unwrap_or(msg.author.id);
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let request = reqwest::get(format!(
        "{base}/api/rank/{guildid}/{userid}",
        base = *crate::shared::BASE_URL,
        guildid = msg.guild_id.unwrap().0,
        userid = usr.0,
    ))
    .await?;
    let data = request.bytes().await?;

    typing.stop();
    let _res = msg
        .channel_id
        .send_files(&ctx.http, std::iter::once((&data[..], "rank.png")), |a| a)
        .await?;

    Ok(())
}
