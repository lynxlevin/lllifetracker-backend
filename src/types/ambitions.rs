use entities::{ambition, prelude::Ambition};
use sea_orm::{DerivePartialModel, FromQueryResult};

use super::{desired_states::DesiredStateVisibleWithActions, ActionVisibleForLinking};

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
    pub desired_state_id: Option<uuid::Uuid>,
    pub desired_state_name: Option<String>,
    pub desired_state_description: Option<String>,
    pub desired_state_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub desired_state_updated_at: Option<chrono::DateTime<chrono::FixedOffset>>,
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
    pub desired_states: Vec<DesiredStateVisibleWithActions>,
}

impl AmbitionVisibleWithLinks {
    pub fn push_desired_state(&mut self, desired_state: DesiredStateVisibleWithActions) {
        self.desired_states.push(desired_state);
    }

    pub fn push_action(&mut self, action: ActionVisibleForLinking) {
        self.desired_states.last_mut().unwrap().push_action(action);
    }
}
