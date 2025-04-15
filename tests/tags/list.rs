use actix_web::{http, test, HttpMessage};
use sea_orm::{entity::prelude::ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use test_utils::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
    let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    let (desired_state, desired_state_tag) =
        factory::desired_state(user.id).insert_with_tag(&db).await?;
    let _archived_action = factory::action(user.id)
        .archived(true)
        .insert_with_tag(&db)
        .await?;
    let _archived_ambition = factory::ambition(user.id)
        .archived(true)
        .insert_with_tag(&db)
        .await?;
    let _archived_desired_state = factory::desired_state(user.id)
        .archived(true)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get().uri("/api/tags").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<TagVisible> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 3);

    let expected = serde_json::json!([
        {
            "id": ambition_tag.id,
            "name": ambition.name.clone(),
            "tag_type": TagType::Ambition,
            "created_at": ambition_tag.created_at,
        },
        {
            "id": desired_state_tag.id,
            "name": desired_state.name.clone(),
            "tag_type": TagType::DesiredState,
            "created_at": desired_state_tag.created_at,
        },
        {
            "id": action_tag.id,
            "name": action.name.clone(),
            "tag_type": TagType::Action,
            "created_at": action_tag.created_at,
        },
    ]);

    let body = serde_json::to_value(&body).unwrap();
    assert_eq!(expected, body);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get().uri("/api/tags").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
