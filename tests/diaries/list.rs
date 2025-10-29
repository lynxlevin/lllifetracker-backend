use actix_web::{http, test, HttpMessage};
use chrono::{Duration, Utc};
use entities::sea_orm_active_enums::TagType;
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::{journal::diaries::types::DiaryVisibleWithTags, tags::types::TagVisible};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let now = Utc::now();
    let diary_0 = factory::diary(user.id).text(None).insert(&db).await?;
    let diary_1 = factory::diary(user.id)
        .date((now - Duration::days(1)).date_naive())
        .insert(&db)
        .await?;
    let plain_tag = factory::tag(user.id).insert(&db).await?;
    let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
    let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    let (desired_state, desired_state_tag) =
        factory::desired_state(user.id).insert_with_tag(&db).await?;
    factory::link_diary_tag(&db, diary_0.id, plain_tag.id).await?;
    factory::link_diary_tag(&db, diary_0.id, ambition_tag.id).await?;
    factory::link_diary_tag(&db, diary_1.id, desired_state_tag.id).await?;
    factory::link_diary_tag(&db, diary_1.id, action_tag.id).await?;

    let req = test::TestRequest::get().uri("/api/diaries").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<DiaryVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        DiaryVisibleWithTags {
            id: diary_0.id,
            text: diary_0.text.clone(),
            date: diary_0.date,
            tags: vec![
                TagVisible {
                    id: ambition_tag.id,
                    name: ambition.name,
                    r#type: TagType::Ambition,
                    created_at: ambition_tag.created_at,
                },
                TagVisible {
                    id: plain_tag.id,
                    name: plain_tag.name.unwrap(),
                    r#type: TagType::Plain,
                    created_at: plain_tag.created_at,
                },
            ],
        },
        DiaryVisibleWithTags {
            id: diary_1.id,
            text: diary_1.text.clone(),
            date: diary_1.date,
            tags: vec![
                TagVisible {
                    id: desired_state_tag.id,
                    name: desired_state.name,
                    r#type: TagType::DesiredState,
                    created_at: desired_state_tag.created_at,
                },
                TagVisible {
                    id: action_tag.id,
                    name: action.name,
                    r#type: TagType::Action,
                    created_at: action_tag.created_at,
                },
            ],
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
async fn multiple_on_same_day_with_tags_should_not_return_duplicates() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let diary_0 = factory::diary(user.id).insert(&db).await?;
    let diary_1 = factory::diary(user.id).insert(&db).await?;
    let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
    let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    let (desired_state, desired_state_tag) =
        factory::desired_state(user.id).insert_with_tag(&db).await?;
    factory::link_diary_tag(&db, diary_0.id, ambition_tag.id).await?;
    factory::link_diary_tag(&db, diary_1.id, desired_state_tag.id).await?;
    factory::link_diary_tag(&db, diary_0.id, action_tag.id).await?;

    let req = test::TestRequest::get().uri("/api/diaries").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<DiaryVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        DiaryVisibleWithTags {
            id: diary_1.id,
            text: diary_1.text.clone(),
            date: diary_1.date,
            tags: vec![TagVisible {
                id: desired_state_tag.id,
                name: desired_state.name,
                r#type: TagType::DesiredState,
                created_at: desired_state_tag.created_at,
            }],
        },
        DiaryVisibleWithTags {
            id: diary_0.id,
            text: diary_0.text.clone(),
            date: diary_0.date,
            tags: vec![
                TagVisible {
                    id: ambition_tag.id,
                    name: ambition.name,
                    r#type: TagType::Ambition,
                    created_at: ambition_tag.created_at,
                },
                TagVisible {
                    id: action_tag.id,
                    name: action.name,
                    r#type: TagType::Action,
                    created_at: action_tag.created_at,
                },
            ],
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
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri("/api/diaries").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
