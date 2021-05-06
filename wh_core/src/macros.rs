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
        return Err($crate::Error::Both(msg: $msg.into(), err: $err.into()).into())
    };
}
