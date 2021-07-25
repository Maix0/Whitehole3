extern crate serenity;
extern crate sqlx;
extern crate wh_core;

pub mod shared;

pub mod module {
    use wh_core::event_handler::WhEventHandlerManager;

    pub static MODULE_DECLARATION: wh_core::ModuleDeclaration = wh_core::ModuleDeclaration {
        command_groups: &[],
        module_name: "Database",
        register_typemap: |t| Box::pin(register_typemap(t)),
        register_event_handler: |e| Box::pin(register_event_handler(e)),
        register_builder,
        register_intent,
    };

    async fn register_event_handler(_: &mut WhEventHandlerManager) {}

    async fn register_typemap(tm: &mut serenity::prelude::TypeMap) {
        let db = sqlx::PgPool::connect(
            std::env::var("DATABASE_URL")
                .expect("Use `DATABASE_URL` environment variable to set the database url")
                .as_str(),
        )
        .await
        .expect("Error when connection to database");

        tm.insert::<crate::shared::DatabaseKey>(db);
    }

    fn register_builder(
        client: serenity::client::ClientBuilder<'_>,
    ) -> serenity::client::ClientBuilder<'_> {
        client
    }

    fn register_intent(
        i: serenity::client::bridge::gateway::GatewayIntents,
    ) -> serenity::client::bridge::gateway::GatewayIntents {
        i
    }
}
