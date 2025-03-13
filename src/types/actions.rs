use entities::{action, prelude::Action};
use sea_orm::{DerivePartialModel, FromQueryResult};

use super::{desired_states::DesiredStateVisibleWithAmbitions, AmbitionVisible};

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "Action")]
pub struct ActionVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trackable: bool,
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
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "Action")]
pub struct ActionVisibleForLinking {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionWithLinksQueryResult {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub desired_state_id: Option<uuid::Uuid>,
    pub desired_state_name: Option<String>,
    pub desired_state_description: Option<String>,
    pub desired_state_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub desired_state_updated_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub ambition_id: Option<uuid::Uuid>,
    pub ambition_name: Option<String>,
    pub ambition_description: Option<String>,
    pub ambition_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub ambition_updated_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ActionVisibleWithLinks {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub desired_states: Vec<DesiredStateVisibleWithAmbitions>,
}

impl ActionVisibleWithLinks {
    pub fn push_desired_state(&mut self, desired_state: DesiredStateVisibleWithAmbitions) {
        self.desired_states.push(desired_state);
    }

    pub fn push_ambition(&mut self, ambition: AmbitionVisible) {
        self.desired_states.last_mut().unwrap().push_ambition(ambition);
    }
}
