use actix_web::{http, test, HttpMessage};
use entities::tag;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let tag = factory::tag(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/tags/plain/{}", tag.id))
        .to_request();
    req.extensions_mut().insert(user);

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
    assert!(tag_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn bad_request_if_not_plain_tag() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/tags/plain/{}", ambition_tag.id))
        .to_request();
    req.extensions_mut().insert(user);

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

    let tag_in_db = tag::Entity::find_by_id(ambition_tag.id).one(&db).await?;
    assert!(tag_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn no_content_without_deletion_on_different_users_tag() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let another_user = factory::user().insert(&db).await?;
    let another_user_tag = factory::tag(another_user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/tags/plain/{}", another_user_tag.id))
        .to_request();
    req.extensions_mut().insert(user);

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let tag_in_db = tag::Entity::find_by_id(another_user_tag.id).one(&db).await?;
    assert!(tag_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn no_content_on_non_existent_tag_id() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/tags/plain/{}", uuid::Uuid::now_v7()))
        .to_request();
    req.extensions_mut().insert(user);

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let tag = factory::tag(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/tags/plain/{}", tag.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
