use crate::entities::{
    ambitions_objectives, book_excerpts_tags, memos_tags, mission_memos_tags, objectives_actions,
};
use sea_orm::{prelude::*, DbConn, DbErr, Set};
use uuid::Uuid;


#[cfg(test)]
pub async fn link_ambition_objective(db: &DbConn, ambition_id: Uuid, objective_id: Uuid) -> Result<ambitions_objectives::Model, DbErr> {
    ambitions_objectives::ActiveModel {
        ambition_id: Set(ambition_id),
        objective_id: Set(objective_id),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn link_objective_action(db: &DbConn, objective_id: Uuid, action_id: Uuid) -> Result<objectives_actions::Model, DbErr> {
    objectives_actions::ActiveModel {
        objective_id: Set(objective_id),
        action_id: Set(action_id),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn link_memo_tag(db: &DbConn, memo_id: Uuid, tag_id: Uuid) -> Result<memos_tags::Model, DbErr> {
    memos_tags::ActiveModel {
        memo_id: Set(memo_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn link_mission_memo_tag(db: &DbConn, mission_memo_id: Uuid, tag_id: Uuid) -> Result<mission_memos_tags::Model, DbErr> {
    mission_memos_tags::ActiveModel {
        mission_memo_id: Set(mission_memo_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn link_book_excerpt_tag(db: &DbConn, book_excerpt_id: Uuid, tag_id: Uuid) -> Result<book_excerpts_tags::Model, DbErr> {
    book_excerpts_tags::ActiveModel {
        book_excerpt_id: Set(book_excerpt_id),
        tag_id: Set(tag_id),
    }
    .insert(db)
    .await
}
