use actix_web::{http, test, HttpMessage};
use entities::{thinking_note, thinking_note_tags};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter, QueryFilter,
    QuerySelect,
};
use use_cases::journal::thinking_notes::types::{ThinkingNoteUpdateRequest, ThinkingNoteVisible};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

    let form = ThinkingNoteUpdateRequest {
        question: Some("New Question to Solve".to_string()),
        thought: Some("I'm thinking.".to_string()),
        answer: Some("Maybe A? Not sure yet.".to_string()),
        tag_ids: vec![ambition_tag.id],
        resolved_at: None,
    };

    let req = test::TestRequest::put()
        .uri(&format!("/api/thinking_notes/{}", thinking_note.id))
        .set_json(&form)
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ThinkingNoteVisible = test::read_body_json(res).await;
    assert_eq!(res.question, form.question);
    assert_eq!(res.thought, form.thought);
    assert_eq!(res.answer, form.answer);
    assert_eq!(res.resolved_at, form.resolved_at);
    assert_eq!(res.created_at, thinking_note.created_at);
    assert!(res.updated_at > thinking_note.updated_at);

    let thinking_note_in_db = thinking_note::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(thinking_note_in_db.user_id, user.id);
    assert_eq!(ThinkingNoteVisible::from(thinking_note_in_db), res);

    let linked_tag_ids: Vec<uuid::Uuid> = thinking_note_tags::Entity::find()
        .column_as(thinking_note_tags::Column::TagId, QueryAs::TagId)
        .filter(thinking_note_tags::Column::ThinkingNoteId.eq(res.id))
        .into_values::<_, QueryAs>()
        .all(&db)
        .await?;
    assert_eq!(linked_tag_ids.len(), 1);
    assert!(linked_tag_ids.contains(&ambition_tag.id));

    Ok(())
}

#[actix_web::test]
async fn not_found_if_invalid_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/thinking_notes/{}", uuid::Uuid::now_v7()))
        .set_json(ThinkingNoteUpdateRequest::default())
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let thinking_note = factory::thinking_note(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/thinking_notes/{}", thinking_note.id))
        .set_json(ThinkingNoteUpdateRequest::default())
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_non_existent_tag_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let thinking_note = factory::thinking_note(user.id).insert(&db).await?;

    let non_existent_tag_req = test::TestRequest::put()
        .uri(&format!("/api/thinking_notes/{}", thinking_note.id))
        .set_json(ThinkingNoteUpdateRequest {
            tag_ids: vec![Uuid::now_v7()],
            ..Default::default()
        })
        .to_request();
    non_existent_tag_req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, non_existent_tag_req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}
