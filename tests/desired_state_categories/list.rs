use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let category_0 = factory::desired_state_category(user.id)
        .ordering(Some(2))
        .insert(&db)
        .await?;
    let category_1 = factory::desired_state_category(user.id)
        .ordering(Some(1))
        .insert(&db)
        .await?;
    let category_2 = factory::desired_state_category(user.id)
        .ordering(None)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/desired_state_categories")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<DesiredStateCategoryVisible> = test::read_body_json(resp).await;
    let expected = vec![
        DesiredStateCategoryVisible::from(category_1),
        DesiredStateCategoryVisible::from(category_0),
        DesiredStateCategoryVisible::from(category_2),
    ];

    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/desired_state_categories")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
