#[macro_export]
macro_rules! message_err {
    ($msg:expr) => {
        return Err($crate::Error::Message($msg.into()).into())
    };
}
#[macro_export]
macro_rules! error_err {
    ($err:expr) => {
        return Err($crate::Error::Error($err.into()).into())
    };
}
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

#[macro_export]
macro_rules! reply_message {
    ($ctx:expr, $msg:expr, $message:expr) => {
        let _ = $msg
            .channel($ctx)
            .await
            .unwrap()
            .guild()
            .unwrap()
            .send_message($ctx, |f| f.content($message))
            .await
            .map_err(|e| error!("Error when sending message: {}", e));
    };
}
