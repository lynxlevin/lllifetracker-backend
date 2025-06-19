use sea_orm::{DerivePartialModel, FromQueryResult};

use entities::{action, prelude::Action, sea_orm_active_enums::ActionTrackType};

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "Action")]
pub struct ActionVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trackable: bool,
    pub color: String,
    pub track_type: ActionTrackType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
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

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ActionCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub track_type: ActionTrackType,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ActionUpdateRequest {
    pub name: String,
    pub description: Option<String>,
    pub trackable: Option<bool>,
    pub color: Option<String>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ActionBulkUpdateOrderRequest {
    pub ordering: Vec<uuid::Uuid>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ActionTrackTypeConversionRequest {
    pub track_type: ActionTrackType,
}
