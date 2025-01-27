use crate::entities::prelude::ActionTrack;
use sea_orm::{DerivePartialModel, FromQueryResult};

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "ActionTrack")]
pub struct ActionTrackVisible {
    pub id: uuid::Uuid,
    pub action_id: Option<uuid::Uuid>,
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, FromQueryResult, Debug)]
pub struct ActionTrackWithActionName {
    pub id: uuid::Uuid,
    pub action_id: Option<uuid::Uuid>,
    pub action_name: Option<String>,
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct ActionTrackAggregation {
    pub durations_by_action: Vec<ActionTrackAggregationDuration>,
}


#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct ActionTrackAggregationDuration {
    pub action_id: uuid::Uuid,
    pub duration: i64,
}