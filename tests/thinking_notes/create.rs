use actix_web::{http, test, HttpMessage};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter, QueryFilter,
    QuerySelect,
};
use use_cases::journal::thinking_notes::types::{ThinkingNoteCreateRequest, ThinkingNoteVisible};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{thinking_note, thinking_note_tags};

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (_, tag_0) = factory::action(user.id)
        .name("action_0".to_string())
        .insert_with_tag(&db)
        .await?;
    let (_, tag_1) = factory::action(user.id)
        .name("action_1".to_string())
        .insert_with_tag(&db)
        .await?;

    let question = "New Question to Solve".to_string();
    let thought = "I'm thinking.".to_string();
    let answer = "Maybe A? Not sure yet.".to_string();
    let req = test::TestRequest::post()
        .uri("/api/thinking_notes")
        .set_json(ThinkingNoteCreateRequest {
            question: Some(question.clone()),
            thought: Some(thought.clone()),
            answer: Some(answer.clone()),
            tag_ids: vec![tag_0.id, tag_1.id],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: ThinkingNoteVisible = test::read_body_json(res).await;
    assert_eq!(res.question, Some(question));
    assert_eq!(res.thought, Some(thought));
    assert_eq!(res.answer, Some(answer));

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
    assert_eq!(linked_tag_ids.len(), 2);
    assert!(linked_tag_ids.contains(&tag_0.id));
    assert!(linked_tag_ids.contains(&tag_1.id));

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/thinking_notes")
        .set_json(ThinkingNoteCreateRequest {
            question: None,
            thought: None,
            answer: None,
            tag_ids: vec![],
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_non_existent_tag_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let non_existent_tag_req = test::TestRequest::post()
        .uri("/api/thinking_notes")
        .set_json(ThinkingNoteCreateRequest {
            question: None,
            thought: None,
            answer: None,
            tag_ids: vec![Uuid::now_v7()],
        })
        .to_request();
    non_existent_tag_req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, non_existent_tag_req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}
