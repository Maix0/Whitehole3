extern crate fern;
extern crate serenity;
// extern crate songbird;

pub struct EmptyEventHandler;

impl serenity::client::EventHandler for EmptyEventHandler {}

pub async fn register_typemap(_: &mut serenity::prelude::TypeMap) {}

pub async fn event_handler() -> Option<EmptyEventHandler> {
    None
}

pub fn register_builder(
    client: serenity::client::ClientBuilder<'_>,
) -> serenity::client::ClientBuilder<'_> {
    client
}

#[macro_use]
pub mod macros;

use serenity::{
    client::Context,
    framework::standard::{
        help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::id::UserId,
};

#[derive(Debug, Clone)]
pub enum Error {
    Message(String),
    Error(String),
    Both { msg: String, err: String },
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::Message(_) => Ok(()),
            Error::Error(s) => write!(f, "{}", s),
            Error::Both { err, .. } => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

#[help]
async fn help_command(
    context: &Context,
    msg: &serenity::model::channel::Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: std::collections::HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
