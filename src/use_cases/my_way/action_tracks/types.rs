use chrono::{DateTime, FixedOffset};
use entities::{action_track, prelude::ActionTrack};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "ActionTrack")]
pub struct ActionTrackVisible {
    pub id: uuid::Uuid,
    pub action_id: uuid::Uuid,
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration: Option<i64>,
}

impl From<action_track::Model> for ActionTrackVisible {
    fn from(item: action_track::Model) -> Self {
        ActionTrackVisible::from(&item)
    }
}

impl From<&action_track::Model> for ActionTrackVisible {
    fn from(item: &action_track::Model) -> Self {
        ActionTrackVisible {
            id: item.id,
            action_id: item.action_id,
            started_at: item.started_at,
            ended_at: item.ended_at,
            duration: item.duration,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ActionTrackListQuery {
    pub active_only: Option<bool>,
    pub started_at_gte: Option<DateTime<FixedOffset>>,
}

#[derive(Deserialize, Debug)]
pub struct ActionTrackAggregationQuery {
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ActionTrackAggregation {
    pub durations_by_action: Vec<ActionTrackAggregationDuration>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ActionTrackAggregationDuration {
    pub action_id: uuid::Uuid,
    pub duration: i64,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionTrackCreateRequest {
    pub started_at: DateTime<FixedOffset>,
    pub action_id: uuid::Uuid,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionTrackUpdateRequest {
    pub action_id: uuid::Uuid,
    pub started_at: DateTime<FixedOffset>,
    pub ended_at: Option<DateTime<FixedOffset>>,
}
