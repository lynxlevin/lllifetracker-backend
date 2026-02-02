use sea_orm::{DerivePartialModel, FromQueryResult};

use entities::{direction_category, prelude::DirectionCategory};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "DirectionCategory")]
pub struct DirectionCategoryVisible {
    pub id: uuid::Uuid,
    pub name: String,
}

impl From<&direction_category::Model> for DirectionCategoryVisible {
    fn from(item: &direction_category::Model) -> Self {
        DirectionCategoryVisible {
            id: item.id,
            name: item.name.clone(),
        }
    }
}

impl From<direction_category::Model> for DirectionCategoryVisible {
    fn from(item: direction_category::Model) -> Self {
        DirectionCategoryVisible::from(&item)
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DirectionCategoryCreateRequest {
    pub name: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DirectionCategoryUpdateRequest {
    pub name: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DirectionCategoryBulkUpdateOrderingRequest {
    pub ordering: Vec<uuid::Uuid>,
}
