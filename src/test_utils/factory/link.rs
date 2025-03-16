use entities::{
    ambitions_desired_states, challenges_tags, desired_states_actions, diaries_tags, memos_tags, reading_notes_tags
};
use sea_orm::{prelude::*, DbConn, DbErr, Set};
use uuid::Uuid;


pub async fn link_ambition_desired_state(db: &DbConn, ambition_id: Uuid, desired_state_id: Uuid) -> Result<ambitions_desired_states::Model, DbErr> {
    ambitions_desired_states::ActiveModel {
        ambition_id: Set(ambition_id),
        desired_state_id: Set(desired_state_id),
    }
    .insert(db)
    .await
}

pub async fn link_desired_state_action(db: &DbConn, desired_state_id: Uuid, action_id: Uuid) -> Result<desired_states_actions::Model, DbErr> {
    desired_states_actions::ActiveModel {
        desired_state_id: Set(desired_state_id),
        action_id: Set(action_id),
    }
    .insert(db)
    .await
}

pub async fn link_memo_tag(db: &DbConn, memo_id: Uuid, tag_id: Uuid) -> Result<memos_tags::Model, DbErr> {
    memos_tags::ActiveModel {
        memo_id: Set(memo_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}

pub async fn link_challenge_tag(db: &DbConn, challenge_id: Uuid, tag_id: Uuid) -> Result<challenges_tags::Model, DbErr> {
    challenges_tags::ActiveModel {
        challenge_id: Set(challenge_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}

pub async fn link_reading_note_tag(db: &DbConn, reading_note_id: Uuid, tag_id: Uuid) -> Result<reading_notes_tags::Model, DbErr> {
    reading_notes_tags::ActiveModel {
        reading_note_id: Set(reading_note_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}

pub async fn link_diary_tag(db: &DbConn, diary_id: Uuid, tag_id: Uuid) -> Result<diaries_tags::Model, DbErr> {
    diaries_tags::ActiveModel {
        diary_id: Set(diary_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}
