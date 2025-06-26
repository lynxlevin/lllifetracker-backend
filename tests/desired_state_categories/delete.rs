use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;
use entities::desired_state_category;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let category = factory::desired_state_category(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/desired_state_categories/{}", category.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let category_in_db = desired_state_category::Entity::find_by_id(category.id)
        .one(&db)
        .await?;
    assert!(category_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn do_nothing_for_other_user_category() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let other_user_category = factory::desired_state_category(other_user.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/desired_state_categories/{}",
            other_user_category.id
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let category_in_db = desired_state_category::Entity::find_by_id(other_user_category.id)
        .one(&db)
        .await?;
    assert!(category_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, ..} = init_app().await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/desired_state_categories/{}", Uuid::now_v7()))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
