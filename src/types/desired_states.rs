use sea_orm::{DerivePartialModel, FromQueryResult};

use super::{ActionVisibleForLinking, AmbitionVisible};
use entities::{desired_state, prelude::DesiredState};

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "DesiredState")]
pub struct DesiredStateVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<desired_state::Model> for DesiredStateVisible {
    fn from(item: desired_state::Model) -> Self {
        DesiredStateVisible {
            id: item.id,
            name: item.name,
            description: item.description,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize)]
pub struct DesiredStateWithLinksQueryResult {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub ambition_id: Option<uuid::Uuid>,
    pub ambition_name: Option<String>,
    pub ambition_description: Option<String>,
    pub ambition_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub ambition_updated_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub action_id: Option<uuid::Uuid>,
    pub action_name: Option<String>,
    pub action_description: Option<String>,
    pub action_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub action_updated_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DesiredStateVisibleWithLinks {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub ambitions: Vec<AmbitionVisible>,
    pub actions: Vec<ActionVisibleForLinking>,
}

impl DesiredStateVisibleWithLinks {
    pub fn push_ambition(&mut self, ambition: AmbitionVisible) {
        self.ambitions.push(ambition);
    }
    pub fn push_action(&mut self, action: ActionVisibleForLinking) {
        self.actions.push(action);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DesiredStateVisibleWithActions {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub actions: Vec<ActionVisibleForLinking>,
}

impl DesiredStateVisibleWithActions {
    pub fn push_action(&mut self, action: ActionVisibleForLinking) {
        self.actions.push(action);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DesiredStateVisibleWithAmbitions {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub ambitions: Vec<AmbitionVisible>,
}

impl DesiredStateVisibleWithAmbitions {
    pub fn push_ambition(&mut self, ambition: AmbitionVisible) {
        self.ambitions.push(ambition);
    }
}
