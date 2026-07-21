use actix_web::{http, test, HttpMessage};
use entities::sea_orm_active_enums::TagType;
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::tags::types::TagVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{
    self, create_actions, create_ambitions, create_directions, create_tags, ActionParam, AmbitionParam,
    DirectionParam, TagParam,
};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let ambitions = create_ambitions(
        vec![
            AmbitionParam {
                name: "ambition_null_ordering",
                ordering: None,
                archived: false,
                ..Default::default()
            },
            AmbitionParam { name: "ambition", ordering: Some(2), archived: false, ..Default::default() },
            AmbitionParam { name: "archived_ambition", ordering: Some(1), archived: true, ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let directions = create_directions(
        vec![
            DirectionParam {
                name: "direction_null_ordering",
                ordering: None,
                archived: false,
                ..Default::default()
            },
            DirectionParam { name: "direction", ordering: Some(2), archived: false, ..Default::default() },
            DirectionParam { name: "archived_direction", ordering: Some(1), archived: true, ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let actions = create_actions(
        vec![
            ActionParam { name: "action_null_ordering", ordering: None, archived: false, ..Default::default() },
            ActionParam { name: "action", ordering: Some(2), archived: false, ..Default::default() },
            ActionParam { name: "archived_action", ordering: Some(1), archived: true, ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let tags = create_tags(
        vec![
            TagParam { name: "plain_tag", r#type: TagType::Plain, ..Default::default() },
            TagParam {
                name: "ambition_null_ordering_tag",
                r#type: TagType::Ambition,
                ambition: Some(ambitions.get("ambition_null_ordering").unwrap()),
                ..Default::default()
            },
            TagParam {
                name: "ambition_tag",
                r#type: TagType::Ambition,
                ambition: Some(ambitions.get("ambition").unwrap()),
                ..Default::default()
            },
            TagParam {
                name: "archived_ambition_tag",
                r#type: TagType::Ambition,
                ambition: Some(ambitions.get("archived_ambition").unwrap()),
                ..Default::default()
            },
            TagParam {
                name: "direction_null_ordering_tag",
                r#type: TagType::Direction,
                direction: Some(directions.get("direction_null_ordering").unwrap()),
                ..Default::default()
            },
            TagParam {
                name: "direction_tag",
                r#type: TagType::Direction,
                direction: Some(directions.get("direction").unwrap()),
                ..Default::default()
            },
            TagParam {
                name: "archived_direction_tag",
                r#type: TagType::Direction,
                direction: Some(directions.get("archived_direction").unwrap()),
                ..Default::default()
            },
            TagParam {
                name: "action_null_ordering_tag",
                r#type: TagType::Action,
                action: Some(actions.get("action_null_ordering").unwrap()),
                ..Default::default()
            },
            TagParam {
                name: "action_tag",
                r#type: TagType::Action,
                action: Some(actions.get("action").unwrap()),
                ..Default::default()
            },
            TagParam {
                name: "archived_action_tag",
                r#type: TagType::Action,
                action: Some(actions.get("archived_action").unwrap()),
                ..Default::default()
            },
        ],
        &user,
        &db,
    )
    .await?;

    let req = test::TestRequest::get().uri("/api/tags").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<TagVisible> = test::read_body_json(resp).await;
    let expected = vec![
        TagVisible::from((
            tags.get("ambition_null_ordering_tag").unwrap(),
            ambitions.get("ambition_null_ordering").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("ambition_tag").unwrap(),
            ambitions.get("ambition").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("archived_ambition_tag").unwrap(),
            ambitions.get("archived_ambition").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("direction_null_ordering_tag").unwrap(),
            directions.get("direction_null_ordering").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("direction_tag").unwrap(),
            directions.get("direction").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("archived_direction_tag").unwrap(),
            directions.get("archived_direction").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("action_null_ordering_tag").unwrap(),
            actions.get("action_null_ordering").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("action_tag").unwrap(),
            actions.get("action").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("archived_action_tag").unwrap(),
            actions.get("archived_action").unwrap().name.clone(),
        )),
        TagVisible::from((
            tags.get("plain_tag").unwrap(),
            tags.get("plain_tag").unwrap().name.clone().unwrap(),
        )),
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
