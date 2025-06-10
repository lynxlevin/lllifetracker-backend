use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use super::super::utils::init_app;
use entities::{ambition, tag};
use common::factory;
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let name = "Test create_ambition route".to_string();
    let description = Some("Test description".to_string());
    let req = test::TestRequest::post()
        .uri("/api/ambitions")
        .set_json(AmbitionCreateRequest {
            name: name.clone(),
            description: description.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: AmbitionVisible = test::read_body_json(res).await;
    assert_eq!(res.name, name.clone());
    assert_eq!(res.description, description.clone());

    let ambition_in_db = ambition::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(ambition_in_db.user_id, user.id);
    assert_eq!(AmbitionVisible::from(ambition_in_db), res);

    let tag_in_db = tag::Entity::find()
        .filter(tag::Column::UserId.eq(user.id))
        .filter(tag::Column::AmbitionId.eq(res.id))
        .filter(tag::Column::DesiredStateId.is_null())
        .filter(tag::Column::ActionId.is_null())
        .one(&db)
        .await?;
    assert!(tag_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn happy_path_no_description() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let name = "Test create_ambition route".to_string();
    let req = test::TestRequest::post()
        .uri("/api/ambitions")
        .set_json(AmbitionCreateRequest {
            name: name.clone(),
            description: None,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: AmbitionVisible = test::read_body_json(res).await;
    assert_eq!(res.name, name.clone());
    assert!(res.description.is_none());

    let ambition_in_db = ambition::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(ambition_in_db.user_id, user.id);
    assert_eq!(AmbitionVisible::from(ambition_in_db), res);

    let tag_in_db = tag::Entity::find()
        .filter(tag::Column::UserId.eq(user.id))
        .filter(tag::Column::AmbitionId.eq(res.id))
        .filter(tag::Column::DesiredStateId.is_null())
        .filter(tag::Column::ActionId.is_null())
        .one(&db)
        .await?;
    assert!(tag_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/ambitions")
        .set_json(AmbitionCreateRequest {
            name: "Test create_ambition not logged in".to_string(),
            description: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
