use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::utils::Connections;

use super::super::utils::init_app;
use entities::{direction, tag};
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (direction, tag) = factory::direction(user.id).insert_with_tag(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/directions/{}", direction.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let direction_in_db = direction::Entity::find_by_id(direction.id)
        .one(&db)
        .await?;
    assert!(direction_in_db.is_none());

    let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
    assert!(tag_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let direction = factory::direction(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/directions/{}", direction.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
