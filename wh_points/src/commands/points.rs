use serenity::client::Context;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::channel::Message;

use points_cmd::*;
use role_cmd::ROLE_COMMAND;

#[command]
#[only_in(guilds)]
#[sub_commands(add, remove, set, role)]
/// The top level command used to manage users and roles points
pub async fn points(ctx: &Context, msg: &Message) -> CommandResult {
    reply_message!(
        ctx,
        msg,
        "This command is divided into differents subcommands: `add`, `remove`, `set` and `role`"
    );

    Ok(())
}

mod points_cmd {
    use serenity::client::Context;
    use serenity::framework::standard::{macros::*, Args, CommandResult};
    use serenity::model::channel::Message;

    #[command]
    #[only_in(guilds)]
    #[usage("[@user] [points]")]
    #[example("@-|Maix| 20")]
    #[num_args(2)]
    /// Add `points` points to the mentioned user
    pub async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let user = msg.mentions.first();
        if user.is_none() {
            message_err!(fluent!(POINTS_ARG_err_user_missing_mention));
        }
        let user = user.unwrap();
        args.advance();
        let points = args.single::<u32>();
        if points.is_err() {
            message_err!(fluent!(POINTS_ARG_err_invalid_number))
        }
        let points: i64 = points.unwrap().into();

        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

        crate::shared::create_user_if_not_exist(ctx, user.id.0, msg.guild_id.unwrap().0).await?;
        let _ = query!("UPDATE user_points SET points = LEAST(points + $1::int8, 4294967295) WHERE guildid = $2::int8 AND userid = $3::int8", points,
        wh_database::shared::Id(msg.guild_id.unwrap().0) as _ ,
        wh_database::shared::Id(user.id.0) as _
    ).execute(db).await?;

        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[usage("[@user] [points]")]
    #[example("@-|Maix| 20")]
    #[num_args(2)]
    /// Remove `points` points of the mentioned user
    pub async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let user = msg.mentions.first();
        if user.is_none() {
            message_err!(fluent!(POINTS_ARG_err_user_missing_mention));
        }
        let user = user.unwrap();
        args.advance();
        let points = args.single::<u32>();
        if points.is_err() {
            message_err!(fluent!(POINTS_ARG_err_invalid_number))
        }
        let points: i64 = points.unwrap().into();

        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

        crate::shared::create_user_if_not_exist(ctx, user.id.0, msg.guild_id.unwrap().0).await?;
        let _ = query!("UPDATE user_points SET points = GREATEST(points - $1::int8, 0) WHERE guildid = $2::int8 AND userid = $3::int8", points,
        wh_database::shared::Id(msg.guild_id.unwrap().0) as _ ,
        wh_database::shared::Id(user.id.0) as _
    ).execute(db).await?;

        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[usage("[@user] [points]")]
    #[example("@-|Maix| 20")]
    #[num_args(2)]
    /// Set the mentioned user points to `points`
    pub async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let user = msg.mentions.first();
        if user.is_none() {
            message_err!(fluent!(POINTS_ARG_err_user_missing_mention));
        }
        let user = user.unwrap();
        args.advance();
        let points = args.single::<u32>();
        if points.is_err() {
            message_err!(fluent!(POINTS_ARG_err_invalid_number))
        }
        let points: i64 = points.unwrap().into();

        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

        crate::shared::create_user_if_not_exist(ctx, user.id.0, msg.guild_id.unwrap().0).await?;
        let _ = query!("UPDATE user_points SET points = GREATEST($1::int8 , 0) WHERE guildid = $2::int8 AND userid = $3::int8", points,
        wh_database::shared::Id(msg.guild_id.unwrap().0) as _ ,
        wh_database::shared::Id(user.id.0) as _
    ).execute(db).await?;

        Ok(())
    }
}

mod role_cmd {

    use serenity::client::Context;
    use serenity::framework::standard::{macros::*, Args, CommandResult};
    use serenity::model::channel::Message;

    #[command]
    #[only_in(guilds)]
    #[sub_commands(new, set, remove)]
    /// The top level command used to manage the roles points
    pub async fn role(ctx: &Context, msg: &Message) -> CommandResult {
        reply_message!(
            ctx,
            msg,
            "This command is divided into differents subcommands: `new`, `set` and `remove`"
        );
        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[usage("@role points")]
    #[example("@role 20")]
    #[num_args(2)]
    /// Create a new role that will be given when the users get to `points` points
    pub async fn new(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let role = msg.mention_roles.first();
        if role.is_none() {
            message_err!(fluent!(POINTS_ARG_err_role_mention_missing))
        }
        let role = role.unwrap();
        args.advance();
        let points = args.single::<u32>();
        if points.is_err() {
            message_err!(fluent!(POINTS_ARG_err_invalid_number));
        }
        let points = points.unwrap();

        let role_db = crate::shared::get_role_points(ctx, msg.guild_id.unwrap().0, role.0).await?;
        if role_db.is_some() {
            message_err!(fluent!(POINTS_role_exists))
        }
        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

        let _ = query!("INSERT INTO role_points (roleid, guildid, points) VALUES ($1::int8, $2::int8, $3::int8)", wh_database::shared::Id(role.0) as _, wh_database::shared::Id(msg.guild_id.unwrap().0) as _, i64::from(points))
        .execute(db).await?;

        reply_message!(ctx, msg, fluent!(POINTS_role_creation));

        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[usage("@role points")]
    #[example("@role 20")]
    #[num_args(2)]
    /// Set the points requierment for a role that has been created
    pub async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let role = msg.mention_roles.first();
        if role.is_none() {
            message_err!(fluent!(POINTS_ARG_err_role_mention_missing))
        }
        let role = role.unwrap();
        let role_db = crate::shared::get_role_points(ctx, msg.guild_id.unwrap().0, role.0).await?;
        if role_db.is_none() {
            message_err!(fluent!(POINTS_role_dont_exists));
        }
        args.advance();
        let points = 0;

        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

        let _ = query!("UPDATE role_points SET points = $1::int8 WHERE roleid = $2::int8 AND guildid = $3::int8", 
        points,wh_database::shared::Id(role.0) as _,wh_database::shared::Id(msg.guild_id.unwrap().0) as _).execute(db).await?;

        Ok(())
    }

    #[command]
    #[only_in(guilds)]
    #[usage("@role")]
    #[example("@role")]
    #[num_args(1)]
    /// Remove the points requirement from a role
    pub async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
        let role = msg.mention_roles.first();
        if role.is_none() {
            message_err!(fluent!(POINTS_ARG_err_role_mention_missing))
        }
        let role = role.unwrap();
        let lock = ctx.data.read().await;
        let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

        let res = query!(
            "DELETE FROM role_points WHERE guildid = $1::int8 AND roleid = $2::int8",
            wh_database::shared::Id(msg.guild_id.unwrap().0) as _,
            wh_database::shared::Id(role.0) as _
        )
        .execute(db)
        .await?;

        if res.rows_affected() == 0 {
            message_err!(fluent!(POINTS_failed_delete_role));
        } else {
            reply_message!(ctx, msg, fluent!(POINTS_success_delete_role));
        }

        Ok(())
    }
}
