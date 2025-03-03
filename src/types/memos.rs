use entities::{memo, prelude::Memo};
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
    pub favorite: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<memo::Model> for MemoVisible {
    fn from(item: memo::Model) -> Self {
        MemoVisible {
            id: item.id,
            title: item.title,
            text: item.text,
            date: item.date,
            favorite: item.favorite,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MemoWithTagQueryResult {
    pub id: uuid::Uuid,
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub favorite: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub tag_id: Option<uuid::Uuid>,
    pub tag_ambition_name: Option<String>,
    pub tag_objective_name: Option<String>,
    pub tag_action_name: Option<String>,
    pub tag_created_at: Option<chrono::DateTime<chrono::FixedOffset>>
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoVisibleWithTags {
    pub id: uuid::Uuid,
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub favorite: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub tags: Vec<TagVisible>,
}

impl MemoVisibleWithTags {
    pub fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }
}
