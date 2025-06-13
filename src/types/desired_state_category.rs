use sea_orm::{DerivePartialModel, FromQueryResult};

use entities::{desired_state_category, prelude::DesiredStateCategory};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "DesiredStateCategory")]
pub struct DesiredStateCategoryVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub ordering: Option<i32>,
}

impl From<&desired_state_category::Model> for DesiredStateCategoryVisible {
    fn from(item: &desired_state_category::Model) -> Self {
        DesiredStateCategoryVisible {
            id: item.id,
            name: item.name.clone(),
            ordering: item.ordering,
        }
    }
}

impl From<desired_state_category::Model> for DesiredStateCategoryVisible {
    fn from(item: desired_state_category::Model) -> Self {
        DesiredStateCategoryVisible::from(&item)
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DesiredStateCategoryCreateRequest {
    pub name: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DesiredStateCategoryUpdateRequest {
    pub name: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DesiredStateCategoryBulkUpdateOrderingRequest {
    pub ordering: Vec<uuid::Uuid>,
}
