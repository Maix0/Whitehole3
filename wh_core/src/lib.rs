extern crate fern;
extern crate serenity;

pub mod event_handler;
#[macro_use]
pub mod macros;

pub struct ModuleDeclaration {
    pub module_name: &'static str,
    pub command_groups: &'static [&'static serenity::framework::standard::CommandGroup],
    pub register_typemap:
        fn(
            &mut serenity::prelude::TypeMap,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + '_ + Send>>,
    pub register_event_handler:
        fn(
            &mut crate::event_handler::WhEventHandlerManager,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + '_ + Send>>,
    pub register_builder:
        fn(serenity::client::ClientBuilder<'_>) -> serenity::client::ClientBuilder<'_>,
    pub register_intent: fn(
        serenity::client::bridge::gateway::GatewayIntents,
    ) -> serenity::client::bridge::gateway::GatewayIntents,
    pub register_init: fn(),
}

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
