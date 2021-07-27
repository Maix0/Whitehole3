use serenity::model::id::UserId;
use serenity::prelude::TypeMapKey;

pub const MAX_QUEUED_ITEM: usize = 250;
pub const TIME_BEFORE_LEAVE: u64 = 5 * 60 * 10u64.pow(3);

#[derive(Clone, Debug)]
pub struct TrackMetadataKey;

impl TypeMapKey for TrackMetadataKey {
    type Value = TrackMetadata;
}

#[derive(Clone, Debug)]
pub struct TrackMetadata {
    pub url: Option<String>,
    pub duration: Option<std::time::Duration>,
    pub title: Option<String>,
    pub added_by: UserId,
}

pub struct MusicEventHandler {
    pub(crate) call: std::sync::Arc<tokio::sync::Mutex<songbird::Call>>,
}

#[serenity::async_trait]
impl songbird::events::EventHandler for MusicEventHandler {
    async fn act(&self, _: &songbird::events::EventContext<'_>) -> Option<songbird::events::Event> {
        if self.call.lock().await.queue().len() == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(TIME_BEFORE_LEAVE)).await;
            if self.call.lock().await.queue().len() == 0 {
                match self.call.lock().await.leave().await {
                    Ok(_) => (),
                    Err(e) => error!("Error when disconnecting: {}", e),
                };
            }
        }
        None
    }
}
