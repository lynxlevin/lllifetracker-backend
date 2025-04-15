use sea_orm::{DerivePartialModel, FromQueryResult};

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

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct DesiredStateCreateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct DesiredStateUpdateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct DesiredStateBulkUpdateOrderingRequest {
    pub ordering: Vec<uuid::Uuid>,
}
