use sea_orm::{DerivePartialModel, FromQueryResult};

use super::{ActionVisible, AmbitionVisible};
use crate::entities::prelude::Objective;

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "Objective")]
pub struct ObjectiveVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize)]
pub struct ObjectiveWithLinksQueryResult {
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
pub struct ObjectiveVisibleWithLinks {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub ambitions: Vec<AmbitionVisible>,
    pub actions: Vec<ActionVisible>,
}

impl ObjectiveVisibleWithLinks {
    pub fn push_ambition(&mut self, ambition: AmbitionVisible) {
        self.ambitions.push(ambition);
    }
    pub fn push_action(&mut self, action: ActionVisible) {
        self.actions.push(action);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ObjectiveVisibleWithActions {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub actions: Vec<ActionVisible>,
}

impl ObjectiveVisibleWithActions {
    pub fn push_action(&mut self, action: ActionVisible) {
        self.actions.push(action);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ObjectiveVisibleWithAmbitions {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub ambitions: Vec<AmbitionVisible>,
}

impl ObjectiveVisibleWithAmbitions {
    pub fn push_ambition(&mut self, ambition: AmbitionVisible) {
        self.ambitions.push(ambition);
    }
}
