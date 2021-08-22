pub static MODULE_DECLARATION: wh_core::ModuleDeclaration = wh_core::ModuleDeclaration {
    module_name: "Points",
    command_groups: &[
        &crate::commands::POINTSMANAGE_GROUP,
        &crate::commands::POINTS_GROUP,
    ],
    register_typemap: |t| Box::pin(register_typemap(t)),
    register_event_handler: |e| Box::pin(register_event_handler(e)),
    register_builder,
    register_intent,
    register_init,
};

async fn register_typemap(tm: &mut serenity::prelude::TypeMap) {
    tm.insert::<crate::shared::TimeMapkey>(crate::shared::TimeMap::new(250));
}

async fn register_event_handler(eh: &mut wh_core::event_handler::WhEventHandlerManager) {
    eh.push(crate::event_handler::PointEventHandler);
}

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

fn register_init() {
    wh_permission::shared::user_permission::add_permission(&["points.manage"])
}
