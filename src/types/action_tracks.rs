use entities::{action_track, prelude::ActionTrack};
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

impl From<action_track::Model> for ActionTrackVisible {
    fn from(item: action_track::Model) -> Self {
        ActionTrackVisible {
            id: item.id,
            action_id: item.action_id,
            started_at: item.started_at,
            ended_at: item.ended_at,
            duration: item.duration,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, FromQueryResult, Debug)]
pub struct ActionTrackWithAction {
    pub id: uuid::Uuid,
    pub action_id: Option<uuid::Uuid>,
    pub action_name: Option<String>,
    pub action_color: Option<String>,
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