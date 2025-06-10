use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use entities::{mindset, tag};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (mindset, tag) = factory::mindset(user.id).insert_with_tag(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/mindsets/{}", mindset.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let mindset_in_db = mindset::Entity::find_by_id(mindset.id).one(&db).await?;
    assert!(mindset_in_db.is_none());

    let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
    assert!(tag_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset = factory::mindset(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/mindsets/{}", mindset.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
