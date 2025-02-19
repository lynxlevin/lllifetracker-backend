use crate::entities::{action, prelude::Action};
use sea_orm::{DerivePartialModel, FromQueryResult};

use super::{objectives::ObjectiveVisibleWithAmbitions, AmbitionVisible};

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
    pub objective_id: Option<uuid::Uuid>,
    pub objective_name: Option<String>,
    pub objective_description: Option<String>,
    pub objective_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub objective_updated_at: Option<chrono::DateTime<chrono::FixedOffset>>,
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
    pub objectives: Vec<ObjectiveVisibleWithAmbitions>,
}

impl ActionVisibleWithLinks {
    pub fn push_objective(&mut self, objective: ObjectiveVisibleWithAmbitions) {
        self.objectives.push(objective);
    }

    pub fn push_ambition(&mut self, ambition: AmbitionVisible) {
        self.objectives.last_mut().unwrap().push_ambition(ambition);
    }
}
