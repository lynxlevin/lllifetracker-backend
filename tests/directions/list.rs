use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::directions::types::DirectionVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let direction_0 = factory::direction(user.id)
        .name("direction_0".to_string())
        .insert(&db)
        .await?;
    let direction_1 = factory::direction(user.id)
        .name("direction_1".to_string())
        .insert(&db)
        .await?;
    let _archived_direction = factory::direction(user.id)
        .archived(true)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/directions")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<DirectionVisible> = test::read_body_json(resp).await;
    let expected = vec![
        DirectionVisible::from(direction_0),
        DirectionVisible::from(direction_1),
    ];

    assert_eq!(body.len(), expected.len());
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}

#[actix_web::test]
async fn happy_path_show_archived_only() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _direction = factory::direction(user.id).insert(&db).await?;
    let archived_direction = factory::direction(user.id)
        .archived(true)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/directions?show_archived_only=true")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<DirectionVisible> = test::read_body_json(resp).await;
    let expected = vec![DirectionVisible::from(archived_direction)];

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

    let req = test::TestRequest::get()
        .uri("/api/directions")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn ordering_with_category() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let category_0 = factory::direction_category(user.id)
        .ordering(Some(1))
        .insert(&db)
        .await?;
    let category_1 = factory::direction_category(user.id)
        .ordering(Some(2))
        .insert(&db)
        .await?;
    let direction_0 = factory::direction(user.id)
        .ordering(Some(1))
        .category_id(Some(category_0.id))
        .insert(&db)
        .await?;
    let direction_1 = factory::direction(user.id)
        .ordering(Some(3))
        .category_id(Some(category_1.id))
        .insert(&db)
        .await?;
    let direction_2 = factory::direction(user.id)
        .ordering(Some(2))
        .category_id(Some(category_0.id))
        .insert(&db)
        .await?;
    let new_direction = factory::direction(user.id)
        .category_id(Some(category_0.id))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/directions")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<DirectionVisible> = test::read_body_json(resp).await;
    let expected = vec![
        DirectionVisible::from(new_direction),
        DirectionVisible::from(direction_0),
        DirectionVisible::from(direction_2),
        DirectionVisible::from(direction_1),
    ];

    assert_eq!(body.len(), expected.len());
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}
