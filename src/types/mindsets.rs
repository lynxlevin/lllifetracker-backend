use sea_orm::{DerivePartialModel, FromQueryResult};

use entities::{mindset, prelude::Mindset};

#[derive(serde::Serialize, serde::Deserialize, FromQueryResult, DerivePartialModel, PartialEq, Debug)]
#[sea_orm(entity="Mindset")]
pub struct MindsetVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<mindset::Model> for MindsetVisible {
    fn from(item: mindset::Model) -> Self {
        MindsetVisible {
            id: item.id,
            name: item.name,
            description: item.description,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct MindsetCreateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct MindsetUpdateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct MindsetBulkUpdateOrderingRequest {
    pub ordering: Vec<uuid::Uuid>,
}
