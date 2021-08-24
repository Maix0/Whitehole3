use serenity::client::Context;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[max_args(1)]
#[usage("[?page]")]
#[example("1")]
/// Show the leaderboard of the current guild at the specified page or if not specified the first page
pub async fn top(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let page = args.single::<u16>().unwrap_or(1);

    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let request = reqwest::get(format!(
        "{base}/api/leaderboard/{guildid}?page={page}",
        base = *crate::shared::BASE_URL,
        guildid = msg.guild_id.unwrap().0,
        page = page
    ))
    .await?;
    let data = request.bytes().await?;

    typing.stop();
    let _res = msg
        .channel_id
        .send_files(
            &ctx.http,
            std::iter::once((&data[..], "leaderbord.png")),
            |a| a,
        )
        .await?;

    Ok(())
}
