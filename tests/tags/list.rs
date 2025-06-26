use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::tags::types::{TagType, TagVisible};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let plain_tag = factory::tag(user.id).insert(&db).await?;
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
    let expected = vec![
        TagVisible {
            id: ambition_tag.id,
            name: ambition.name.clone(),
            tag_type: TagType::Ambition,
            created_at: ambition_tag.created_at,
        },
        TagVisible {
            id: desired_state_tag.id,
            name: desired_state.name.clone(),
            tag_type: TagType::DesiredState,
            created_at: desired_state_tag.created_at,
        },
        TagVisible {
            id: action_tag.id,
            name: action.name.clone(),
            tag_type: TagType::Action,
            created_at: action_tag.created_at,
        },
        TagVisible {
            id: plain_tag.id,
            name: plain_tag.name.unwrap(),
            tag_type: TagType::Plain,
            created_at: plain_tag.created_at,
        },
    ];

    assert_eq!(body.len(), expected.len());
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, ..} = init_app().await?;

    let req = test::TestRequest::get().uri("/api/tags").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
