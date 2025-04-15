use sea_orm::{ActiveModelTrait, DbConn, DbErr, Set};
use uuid::Uuid;

use entities::{diaries_tags, reading_notes_tags};

pub async fn link_reading_note_tag(
    db: &DbConn,
    reading_note_id: Uuid,
    tag_id: Uuid,
) -> Result<reading_notes_tags::Model, DbErr> {
    reading_notes_tags::ActiveModel {
        reading_note_id: Set(reading_note_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}

pub async fn link_diary_tag(
    db: &DbConn,
    diary_id: Uuid,
    tag_id: Uuid,
) -> Result<diaries_tags::Model, DbErr> {
    diaries_tags::ActiveModel {
        diary_id: Set(diary_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}
