use chrono::{DateTime, FixedOffset};
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
    pub is_focused: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<&desired_state::Model> for DesiredStateVisible {
    fn from(item: &desired_state::Model) -> Self {
        DesiredStateVisible {
            id: item.id,
            name: item.name.clone(),
            description: item.description.clone(),
            category_id: item.category_id,
            is_focused: item.is_focused,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

impl From<desired_state::Model> for DesiredStateVisible {
    fn from(item: desired_state::Model) -> Self {
        DesiredStateVisible::from(&item)
    }
}

#[derive(Deserialize, Debug)]
pub struct DesiredStateListQuery {
    pub show_archived_only: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DesiredStateCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub is_focused: bool,
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
