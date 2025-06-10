use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use super::super::utils::init_app;
use common::factory;
use entities::{mindset, tag};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let name = "Test create_mindset route".to_string();
    let description = Some("Test description".to_string());
    let req = test::TestRequest::post()
        .uri("/api/mindsets")
        .set_json(MindsetCreateRequest {
            name: name.clone(),
            description: description.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: MindsetVisible = test::read_body_json(res).await;
    assert_eq!(res.name, name.clone());
    assert_eq!(res.description, description.clone());

    let mindset_in_db = mindset::Entity::find_by_id(res.id).one(&db).await?.unwrap();
    assert_eq!(mindset_in_db.user_id, user.id);
    assert_eq!(MindsetVisible::from(mindset_in_db), res);

    let tag_in_db = tag::Entity::find()
        .filter(tag::Column::UserId.eq(user.id))
        .filter(tag::Column::MindsetId.eq(res.id))
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

    let name = "Test create_mindset route".to_string();
    let req = test::TestRequest::post()
        .uri("/api/mindsets")
        .set_json(MindsetCreateRequest {
            name: name.clone(),
            description: None,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: MindsetVisible = test::read_body_json(res).await;
    assert_eq!(res.name, name.clone());
    assert!(res.description.is_none());

    let mindset_in_db = mindset::Entity::find_by_id(res.id).one(&db).await?.unwrap();
    assert_eq!(mindset_in_db.user_id, user.id);
    assert_eq!(MindsetVisible::from(mindset_in_db), res);

    let tag_in_db = tag::Entity::find()
        .filter(tag::Column::UserId.eq(user.id))
        .filter(tag::Column::MindsetId.eq(res.id))
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
        .uri("/api/mindsets")
        .set_json(MindsetCreateRequest {
            name: "Test create_mindset not logged in".to_string(),
            description: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
