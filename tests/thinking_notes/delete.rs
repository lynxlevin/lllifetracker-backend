use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{thinking_note, thinking_note_tags};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    factory::link_thinking_note_tag(&db, thinking_note.id, ambition_tag.id).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/thinking_notes/{}", thinking_note.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let thinking_note_in_db = thinking_note::Entity::find_by_id(thinking_note.id)
        .one(&db)
        .await?;
    assert!(thinking_note_in_db.is_none());

    let thinking_note_tags_in_db = thinking_note_tags::Entity::find()
        .filter(thinking_note_tags::Column::ThinkingNoteId.eq(thinking_note.id))
        .filter(thinking_note_tags::Column::TagId.eq(ambition_tag.id))
        .one(&db)
        .await?;
    assert!(thinking_note_tags_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let thinking_note = factory::thinking_note(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/thinking_notes/{}", thinking_note.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
