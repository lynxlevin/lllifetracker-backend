use actix_web::{http, test, HttpMessage};
use entities::sea_orm_active_enums::TagType;
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::tags::types::TagVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let plain_tag = factory::tag(user.id).insert(&db).await?;
    let (ambition_null_ordering, ambition_null_ordering_tag) = factory::ambition(user.id)
        .name("ambition_null_ordering".to_string())
        .insert_with_tag(&db)
        .await?;
    let (direction_null_ordering, direction_null_ordering_tag) = factory::direction(user.id)
        .name("direction_null_ordering".to_string())
        .insert_with_tag(&db)
        .await?;
    let (action_null_ordering, action_null_ordering_tag) = factory::action(user.id)
        .name("action_null_ordering".to_string())
        .insert_with_tag(&db)
        .await?;
    let (ambition, ambition_tag) = factory::ambition(user.id)
        .ordering(Some(2))
        .name("ambition".to_string())
        .insert_with_tag(&db)
        .await?;
    let (direction, direction_tag) = factory::direction(user.id)
        .ordering(Some(2))
        .name("direction".to_string())
        .insert_with_tag(&db)
        .await?;
    let (action, action_tag) = factory::action(user.id)
        .ordering(Some(2))
        .name("action".to_string())
        .insert_with_tag(&db)
        .await?;
    let (archived_ambition, archived_ambition_tag) = factory::ambition(user.id)
        .archived(true)
        .name("archived_ambition".to_string())
        .ordering(Some(1))
        .insert_with_tag(&db)
        .await?;
    let (archived_direction, archived_direction_tag) = factory::direction(user.id)
        .archived(true)
        .name("archived_direction".to_string())
        .ordering(Some(1))
        .insert_with_tag(&db)
        .await?;
    let (archived_action, archived_action_tag) = factory::action(user.id)
        .archived(true)
        .name("archived_action".to_string())
        .ordering(Some(1))
        .insert_with_tag(&db)
        .await?;

    let req = test::TestRequest::get().uri("/api/tags").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<TagVisible> = test::read_body_json(resp).await;
    let expected = vec![
        TagVisible {
            id: archived_ambition_tag.id,
            name: archived_ambition.name.clone(),
            r#type: TagType::Ambition,
            created_at: archived_ambition_tag.created_at,
        },
        TagVisible {
            id: ambition_tag.id,
            name: ambition.name.clone(),
            r#type: TagType::Ambition,
            created_at: ambition_tag.created_at,
        },
        TagVisible {
            id: ambition_null_ordering_tag.id,
            name: ambition_null_ordering.name.clone(),
            r#type: TagType::Ambition,
            created_at: ambition_null_ordering_tag.created_at,
        },
        TagVisible {
            id: direction_null_ordering_tag.id,
            name: direction_null_ordering.name.clone(),
            r#type: TagType::Direction,
            created_at: direction_null_ordering_tag.created_at,
        },
        TagVisible {
            id: archived_direction_tag.id,
            name: archived_direction.name.clone(),
            r#type: TagType::Direction,
            created_at: archived_direction_tag.created_at,
        },
        TagVisible {
            id: direction_tag.id,
            name: direction.name.clone(),
            r#type: TagType::Direction,
            created_at: direction_tag.created_at,
        },
        TagVisible {
            id: archived_action_tag.id,
            name: archived_action.name.clone(),
            r#type: TagType::Action,
            created_at: archived_action_tag.created_at,
        },
        TagVisible {
            id: action_tag.id,
            name: action.name.clone(),
            r#type: TagType::Action,
            created_at: action_tag.created_at,
        },
        TagVisible {
            id: action_null_ordering_tag.id,
            name: action_null_ordering.name.clone(),
            r#type: TagType::Action,
            created_at: action_null_ordering_tag.created_at,
        },
        TagVisible {
            id: plain_tag.id,
            name: plain_tag.name.unwrap(),
            r#type: TagType::Plain,
            created_at: plain_tag.created_at,
        },
    ];

    assert_eq!(body.len(), expected.len());
    dbg!(&body);
    dbg!(&expected);
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri("/api/tags").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
