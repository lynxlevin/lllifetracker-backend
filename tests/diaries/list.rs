use actix_web::{http, test, HttpMessage};
use chrono::{Duration, Utc};
use sea_orm::{entity::prelude::ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use test_utils::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let now = Utc::now();
    let diary_0 = factory::diary(user.id).text(None).insert(&db).await?;
    let diary_1 = factory::diary(user.id)
        .date((now - Duration::days(1)).date_naive())
        .insert(&db)
        .await?;
    let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
    let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    let (desired_state, desired_state_tag) =
        factory::desired_state(user.id).insert_with_tag(&db).await?;
    factory::link_diary_tag(&db, diary_0.id, ambition_tag.id).await?;
    factory::link_diary_tag(&db, diary_1.id, desired_state_tag.id).await?;
    factory::link_diary_tag(&db, diary_1.id, action_tag.id).await?;

    let req = test::TestRequest::get().uri("/api/diaries").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<DiaryVisibleWithTags> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 2);

    let expected_0 = serde_json::json!({
        "id": diary_0.id,
        "text": diary_0.text.clone(),
        "date": diary_0.date,
        "score": diary_0.score,
        "tags": [
            {
                "id": ambition_tag.id,
                "name": ambition.name,
                "tag_type": TagType::Ambition,
                "created_at": ambition_tag.created_at,
            },
        ],
    });
    let body_0 = serde_json::to_value(&body[0]).unwrap();
    assert_eq!(expected_0, body_0);

    let expected_1 = serde_json::json!({
        "id": diary_1.id,
        "text": diary_1.text.clone(),
        "date": diary_1.date,
        "score": diary_1.score.clone(),
        "tags": [
            {
                "id": desired_state_tag.id,
                "name": desired_state.name,
                "tag_type": TagType::DesiredState,
                "created_at": desired_state_tag.created_at,
            },
            {
                "id": action_tag.id,
                "name": action.name,
                "tag_type": TagType::Action,
                "created_at": action_tag.created_at,
            },
        ],
    });
    let body_1 = serde_json::to_value(&body[1]).unwrap();
    assert_eq!(expected_1, body_1,);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get().uri("/api/diaries").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
