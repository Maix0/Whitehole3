pub static MODULE_DECLARATION: wh_core::ModuleDeclaration = wh_core::ModuleDeclaration {
    module_name: "Config",
    command_groups: &[],
    register_typemap: |t| Box::pin(register_typemap(t)),
    register_event_handler: |e| Box::pin(register_event_handler(e)),
    register_builder,
    register_intent,
    register_init,
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
    intent
}

fn register_init() {}
