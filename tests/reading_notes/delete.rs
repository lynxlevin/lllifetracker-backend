use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use super::super::utils::init_app;
use entities::{reading_note, reading_notes_tags};
use test_utils::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let reading_note = factory::reading_note(user.id).insert(&db).await?;
    let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    factory::link_reading_note_tag(&db, reading_note.id, ambition_tag.id).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/reading_notes/{}", reading_note.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let reading_note_in_db = reading_note::Entity::find_by_id(reading_note.id)
        .one(&db)
        .await?;
    assert!(reading_note_in_db.is_none());

    let reading_notes_tags_in_db = reading_notes_tags::Entity::find()
        .filter(reading_notes_tags::Column::ReadingNoteId.eq(reading_note.id))
        .filter(reading_notes_tags::Column::TagId.eq(ambition_tag.id))
        .one(&db)
        .await?;
    assert!(reading_notes_tags_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let reading_note = factory::reading_note(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/reading_notes/{}", reading_note.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
