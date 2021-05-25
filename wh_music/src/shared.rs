use serenity::model::id::UserId;
use serenity::prelude::TypeMapKey;

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
