use sea_orm::{DerivePartialModel, FromQueryResult};

use entities::{ambition, prelude::Ambition};

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

impl From<&ambition::Model> for AmbitionVisible {
    fn from(item: &ambition::Model) -> Self {
        AmbitionVisible {
            id: item.id,
            name: item.name.clone(),
            description: item.description.clone(),
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

impl From<ambition::Model> for AmbitionVisible {
    fn from(item: ambition::Model) -> Self {
        AmbitionVisible::from(&item)
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct AmbitionCreateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct AmbitionUpdateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct AmbitionBulkUpdateOrderingRequest {
    pub ordering: Vec<uuid::Uuid>,
}
