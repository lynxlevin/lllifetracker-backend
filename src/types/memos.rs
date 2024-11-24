use crate::entities::prelude::Memo;
use sea_orm::{DerivePartialModel, FromQueryResult};

use super::TagVisible;

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "Memo")]
pub struct MemoVisible {
    pub id: uuid::Uuid,
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoWithTagQueryResult {
    pub id: uuid::Uuid,
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub tag_id: Option<uuid::Uuid>,
    pub tag_ambition_id: Option<uuid::Uuid>,
    pub tag_ambition_name: Option<String>,
    pub tag_objective_id: Option<uuid::Uuid>,
    pub tag_objective_name: Option<String>,
    pub tag_action_id: Option<uuid::Uuid>,
    pub tag_action_name: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoVisibleWithTags {
    pub id: uuid::Uuid,
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub tags: Vec<TagVisible>,
}

impl MemoVisibleWithTags {
    pub fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }
}
