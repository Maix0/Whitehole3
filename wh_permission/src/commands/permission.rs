use serenity::client::Context;
use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::model::channel::Message;

#[command]
#[only_in(guilds)]
#[sub_commands(grant, remove, view)]
pub async fn permission(ctx: &Context, msg: &Message) -> CommandResult {
    reply_message!(
        ctx,
        msg,
        "This command is separated into sub commands: `grant`, `remove`, `view`"
    );
    Ok(())
}

#[command]
#[only_in(guilds)]
#[usage("[@user] [permission]")]
#[example("@-|Maix|#1010 permission.manage")]
pub async fn grant(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_mention = msg.mentions.first();
    if user_mention.is_none() {
        message_err!("You need to mention someone");
    }
    let user_mention = user_mention.unwrap();
    args.advance();
    let permission = args.single::<String>();
    if permission.is_err() {
        message_err!("You need to provide a permission to give!");
    }
    let permission = permission.unwrap();
    if permission == "permission.manage" {
        let discord_permission = msg
            .guild(&ctx.cache)
            .await
            .unwrap()
            .member_permissions(ctx, msg.author.id)
            .await;
        if let Err(e) = &discord_permission {
            both_err!("Internal Error", format!("Internal Error: {}", e));
        }
        let discord_permission = discord_permission.unwrap();
        if !discord_permission.administrator() {
            message_err!("This permission can only be managed by having the ADMINISTRATOR discord permission")
        }
    }
    let lock = ctx.data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
    crate::shared::create_permission_if_not_exists(ctx, user_mention.id.0, msg.guild_id.unwrap().0)
        .await?;
    let res = query!("update permission set ids = array_distinct(array_append(ids, $3::text)) where userid  = $1::int8 and guildid = $2::int8;",
        wh_database::shared::Id(user_mention.id.0) as _,
        wh_database::shared::Id(msg.guild_id.unwrap().0) as _,
        permission
    ).execute(db).await;

    if let Err(e) = &res {
        both_err!(
            "An error occured with the database",
            format!("Error when granting permission: {}", e)
        );
    }

    reply_message!(ctx, msg, "The permission has been granted");
    Ok(())
}

#[command]
#[only_in(guilds)]
#[usage("[@user] [permission]")]
#[example("@-|Maix|#1010 permission.manage")]
pub async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user_mention = msg.mentions.first();
    if user_mention.is_none() {
        message_err!("You need to mention someone");
    }
    let user_mention = user_mention.unwrap();
    args.advance();
    let permission = args.single::<String>();
    if permission.is_err() {
        message_err!("You need to provide a permission to remove!");
    }
    let permission = permission.unwrap();
    if permission == "permission.manage" {
        let discord_permission = msg
            .guild(&ctx.cache)
            .await
            .unwrap()
            .member_permissions(ctx, msg.author.id)
            .await;
        if let Err(e) = &discord_permission {
            both_err!("Internal Error", format!("Internal Error: {}", e));
        }
        let discord_permission = discord_permission.unwrap();
        if !discord_permission.administrator() {
            message_err!("This permission can only be managed by having the ADMINISTRATOR discord permission")
        }
    }
    let lock = ctx.data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
    crate::shared::create_permission_if_not_exists(ctx, user_mention.id.0, msg.guild_id.unwrap().0)
        .await?;
    let res = query!("update permission set ids = array_distinct(array_diff(ids, ARRAY[$3::text])) where userid  = $1::int8 and guildid = $2::int8;",
        wh_database::shared::Id(user_mention.id.0) as _,
        wh_database::shared::Id(msg.guild_id.unwrap().0) as _,
        permission
    ).execute(db).await;

    if let Err(e) = &res {
        both_err!(
            "An error occured with the database",
            format!("Error when removing permission: {}", e)
        );
    }
    reply_message!(ctx, msg, "The permission has been removed");
    Ok(())
}

#[command]
#[only_in(guilds)]
#[usage("[@user]")]
#[example("@-|Maix|#1010")]
#[min_args(0)]
#[max_args(1)]
pub async fn view(ctx: &Context, msg: &Message) -> CommandResult {
    let usr_mention = msg.mentions.first().unwrap_or(&msg.author);
    crate::shared::create_permission_if_not_exists(ctx, usr_mention.id.0, msg.guild_id.unwrap().0)
        .await?;
    let data =
        crate::shared::get_permission(ctx, usr_mention.id.0, msg.guild_id.unwrap().0).await?;
    let data = data.unwrap();
    use serenity::prelude::Mentionable;

    reply_message!(
        ctx,
        msg,
        format!(
            "{} permissions are: {}",
            usr_mention.mention(),
            data.ids
                .iter()
                .map(|p| format!("`{}` ", p))
                .collect::<String>()
        )
    );
    Ok(())
}
