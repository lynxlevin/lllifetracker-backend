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

impl From<action::Model> for ActionVisible {
    fn from(item: action::Model) -> Self {
        ActionVisible {
            id: item.id,
            name: item.name,
            description: item.description,
            trackable: item.trackable,
            color: item.color,
            track_type: item.track_type,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}
