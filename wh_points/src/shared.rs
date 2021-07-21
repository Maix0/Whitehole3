#[derive(Debug, Clone)]
pub struct PointKey;

impl serenity::prelude::TypeMapKey for PointKey {
    type Value = PointDataList;
}

#[derive(Debug, Clone)]
pub struct PointDataList {
    pub last_sync: std::time::Instant,
    pub points: std::collections::HashMap<
        (serenity::model::id::GuildId, serenity::model::id::UserId),
        PointData,
    >,
}

#[derive(Debug, Clone)]
pub struct PointData {
    pub last_time: std::time::Instant,
    pub points: u64,
}

pub const SYNC_EVERY: std::time::Duration =
    std::time::Duration::from_secs(if cfg!(debug_assertions) { 30 } else { 5 * 60 });

pub const DELAY_BETWEEN_MESSAGE: std::time::Duration =
    std::time::Duration::from_secs(if cfg!(debug_assertions) { 30 } else { 2 * 60 });
pub const POINT_RANGE: (u64, u64) = (5, 10);

#[derive(Clone, Debug, sqlx::Type)]
pub struct PointRow {
    uid: wh_database::shared::Id,
}
