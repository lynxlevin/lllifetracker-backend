use chrono::{DateTime, FixedOffset};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::{action, prelude::Action, sea_orm_active_enums::ActionTrackType};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "Action")]
pub struct ActionVisible {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trackable: bool,
    pub color: String,
    pub track_type: ActionTrackType,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<&action::Model> for ActionVisible {
    fn from(item: &action::Model) -> Self {
        ActionVisible {
            id: item.id,
            name: item.name.clone(),
            description: item.description.clone(),
            trackable: item.trackable,
            color: item.color.clone(),
            track_type: item.track_type.clone(),
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

impl From<action::Model> for ActionVisible {
    fn from(item: action::Model) -> Self {
        ActionVisible::from(&item)
    }
}

#[derive(Deserialize, Debug)]
pub struct ActionListQuery {
    pub show_archived_only: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub track_type: ActionTrackType,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionUpdateRequest {
    pub name: String,
    pub description: Option<String>,
    pub trackable: Option<bool>,
    pub color: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionBulkUpdateOrderRequest {
    pub ordering: Vec<uuid::Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionTrackTypeConversionRequest {
    pub track_type: ActionTrackType,
}
