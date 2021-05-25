pub static MODULE_DECLARATION: wh_core::ModuleDeclaration = wh_core::ModuleDeclaration {
    module_name: "Points",
    command_groups: &[],
    register_typemap: |t| Box::pin(register_typemap(t)),
    register_event_handler: |e| Box::pin(register_event_handler(e)),
    register_builder,
    register_intent,
};

async fn register_typemap(_: &mut serenity::prelude::TypeMap) {}

async fn register_event_handler(_: &mut wh_core::event_handler::WhEventHandlerManager) {}

fn register_builder(
    client: serenity::client::ClientBuilder<'_>,
) -> serenity::client::ClientBuilder<'_> {
    client
}

fn register_intent(
    intent: serenity::client::bridge::gateway::GatewayIntents,
) -> serenity::client::bridge::gateway::GatewayIntents {
    use serenity::client::bridge::gateway::GatewayIntents as I;
    intent | I::GUILD_MESSAGES | I::GUILDS | I::GUILD_MESSAGE_REACTIONS
}
