use chrono::{DateTime, FixedOffset};
use sea_orm::{DerivePartialModel, FromQueryResult};

use entities::{ambition, prelude::Ambition};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "Ambition")]
pub struct AmbitionVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
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

#[derive(Deserialize, Debug)]
pub struct AmbitionListQuery {
    pub show_archived_only: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AmbitionCreateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AmbitionUpdateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AmbitionBulkUpdateOrderingRequest {
    pub ordering: Vec<uuid::Uuid>,
}
