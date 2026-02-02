use chrono::{DateTime, FixedOffset};
use sea_orm::{DerivePartialModel, FromQueryResult};

use entities::{direction, prelude::Direction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "Direction")]
pub struct DirectionVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<&direction::Model> for DirectionVisible {
    fn from(item: &direction::Model) -> Self {
        DirectionVisible {
            id: item.id,
            name: item.name.clone(),
            description: item.description.clone(),
            category_id: item.category_id,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

impl From<direction::Model> for DirectionVisible {
    fn from(item: direction::Model) -> Self {
        DirectionVisible::from(&item)
    }
}

#[derive(Deserialize, Debug)]
pub struct DirectionListQuery {
    pub show_archived_only: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DirectionCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DirectionUpdateRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DirectionBulkUpdateOrderingRequest {
    pub ordering: Vec<uuid::Uuid>,
}
