use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::utils::Connections;

use super::super::utils::init_app;
use entities::{ambition, tag};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (ambition, tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/ambitions/{}", ambition.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let ambition_in_db = ambition::Entity::find_by_id(ambition.id).one(&db).await?;
    assert!(ambition_in_db.is_none());

    let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
    assert!(tag_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let ambition = factory::ambition(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/ambitions/{}", ambition.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
