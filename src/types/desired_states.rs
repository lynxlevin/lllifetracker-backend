use sea_orm::{DerivePartialModel, FromQueryResult};

use entities::{desired_state, prelude::DesiredState};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "DesiredState")]
pub struct DesiredStateVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<desired_state::Model> for DesiredStateVisible {
    fn from(item: desired_state::Model) -> Self {
        DesiredStateVisible {
            id: item.id,
            name: item.name,
            description: item.description,
            category_id: item.category_id,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DesiredStateCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DesiredStateUpdateRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DesiredStateBulkUpdateOrderingRequest {
    pub ordering: Vec<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum DesiredStateConvertToType {
    Mindset,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DesiredStateConvertRequest {
    pub convert_to: DesiredStateConvertToType,
}
