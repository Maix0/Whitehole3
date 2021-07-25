#[macro_export]
/// Returns an error that will be sent as a message
/// ```rust
///     message_err!("This will be sent as a discord message");
/// ```
macro_rules! message_err {
    ($msg:expr) => {
        return Err($crate::Error::Message($msg.into()).into())
    };
}

/// Returns an error that will be log in the console
/// ```rust
///     error_err!("This will be log in the console");
/// ```
#[macro_export]
macro_rules! error_err {
    ($err:expr) => {
        return Err($crate::Error::Error($err.into()).into())
    };
}
/// Returns an error with two message, one for a discord message and one for the console
/// ```rust
///    both_err!("This will be logged in a discord message", "This will be logged in the console");
/// ```
#[macro_export]
macro_rules! both_err {
    ($msg:expr, $err:expr) => {
        return Err($crate::Error::Both {
            msg: $msg.into(),
            err: $err.into(),
        }
        .into())
    };
}

/// Reply to the given message ($msg) with the context ($ctx) with the given message ($message)
#[macro_export]
macro_rules! reply_message {
    ($ctx:expr, $msg:expr, $message:expr) => {
        let _ = $msg
            .channel_id
            .send_message(&$ctx.http, |f| f.content($message))
            .await
            .map_err(|e| error!("Error when sending message: {}", e));
    };
}

#[macro_export]
macro_rules! add_commands {
    ($group_name:ident, ($($cmd:ident),* ,($($check:ident),*))) => {
        use serenity::framework::standard::macros::*;

        $(
            mod $cmd;
            use $cmd::*;
        )*

        #[group]
        #[commands($($cmd),*)]
        $(#[checks($check)])*
        #[only_in(guilds)]
        struct $group_name;
    };
}
