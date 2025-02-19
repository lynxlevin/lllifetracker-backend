use crate::entities::{ambition, prelude::Ambition};
use sea_orm::{DerivePartialModel, FromQueryResult};

use super::{objectives::ObjectiveVisibleWithActions, ActionVisibleForLinking};

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "Ambition")]
pub struct AmbitionVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<ambition::Model> for AmbitionVisible {
    fn from(item: ambition::Model) -> Self {
        AmbitionVisible {
            id: item.id,
            name: item.name,
            description: item.description,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmbitionWithLinksQueryResult {
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
    pub action_id: Option<uuid::Uuid>,
    pub action_name: Option<String>,
    pub action_description: Option<String>,
    pub action_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub action_updated_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AmbitionVisibleWithLinks {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub objectives: Vec<ObjectiveVisibleWithActions>,
}

impl AmbitionVisibleWithLinks {
    pub fn push_objective(&mut self, objective: ObjectiveVisibleWithActions) {
        self.objectives.push(objective);
    }

    pub fn push_action(&mut self, action: ActionVisibleForLinking) {
        self.objectives.last_mut().unwrap().push_action(action);
    }
}
