use sea_orm::{DerivePartialModel, FromQueryResult};
use crate::entities::prelude::Action;

#[derive(serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Action")]
pub struct ActionVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}
