use actix_web::{http, test, HttpMessage};
use sea_orm::{
    entity::prelude::ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter,
    QueryFilter, QuerySelect,
};

use super::super::utils::init_app;
use entities::{reading_note, reading_notes_tags};
use test_utils::{self, *};
use types::*;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (_, tag_0) = factory::action(user.id)
        .name("action_0".to_string())
        .insert_with_tag(&db)
        .await?;
    let (_, tag_1) = factory::action(user.id)
        .name("action_1".to_string())
        .insert_with_tag(&db)
        .await?;

    let reading_note_title = "New reading note".to_string();
    let page_number = 12;
    let reading_note_text = "This is a new reading note for testing create method.".to_string();
    let today = chrono::Utc::now().date_naive();
    let req = test::TestRequest::post()
        .uri("/api/reading_notes")
        .set_json(ReadingNoteCreateRequest {
            title: reading_note_title.clone(),
            page_number: page_number,
            text: reading_note_text.clone(),
            date: today,
            tag_ids: vec![tag_0.id, tag_1.id],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: ReadingNoteVisible = test::read_body_json(res).await;
    assert_eq!(res.title, reading_note_title.clone());
    assert_eq!(res.page_number, page_number);
    assert_eq!(res.text, reading_note_text.clone());
    assert_eq!(res.date, today);

    let reading_note_in_db = reading_note::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(reading_note_in_db.user_id, user.id);
    assert_eq!(ReadingNoteVisible::from(reading_note_in_db), res);

    let linked_tag_ids: Vec<uuid::Uuid> = reading_notes_tags::Entity::find()
        .column_as(reading_notes_tags::Column::TagId, QueryAs::TagId)
        .filter(reading_notes_tags::Column::ReadingNoteId.eq(res.id))
        .into_values::<_, QueryAs>()
        .all(&db)
        .await?;
    assert_eq!(linked_tag_ids.len(), 2);
    assert!(linked_tag_ids.contains(&tag_0.id));
    assert!(linked_tag_ids.contains(&tag_1.id));

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/reading_notes")
        .set_json(ReadingNoteCreateRequest {
            title: "New ReadingNote".to_string(),
            page_number: 1,
            text: "This is a new reading note for testing create method.".to_string(),
            date: chrono::Utc::now().date_naive(),
            tag_ids: vec![],
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
