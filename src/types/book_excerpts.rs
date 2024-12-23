use crate::entities::prelude::BookExcerpt;
use sea_orm::{DerivePartialModel, FromQueryResult};

use super::TagVisible;

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "BookExcerpt")]
pub struct BookExcerptVisible {
    pub id: uuid::Uuid,
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BookExcerptWithTagQueryResult {
    pub id: uuid::Uuid,
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub tag_id: Option<uuid::Uuid>,
    pub tag_ambition_name: Option<String>,
    pub tag_objective_name: Option<String>,
    pub tag_action_name: Option<String>,
    pub tag_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BookExcerptVisibleWithTags {
    pub id: uuid::Uuid,
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub tags: Vec<TagVisible>,
}

impl BookExcerptVisibleWithTags {
    pub fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }
}
