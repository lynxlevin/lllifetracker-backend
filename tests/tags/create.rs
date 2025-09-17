use actix_web::{http, test, HttpMessage};
use entities::{sea_orm_active_enums::TagType, tag};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::tags::types::{TagCreateRequest, TagVisible};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req_body = TagCreateRequest {
        name: "new_tag".to_string(),
    };

    let req = test::TestRequest::post()
        .set_json(req_body.clone())
        .uri("/api/tags/plain")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::CREATED);

    let res: TagVisible = test::read_body_json(resp).await;
    assert_eq!(res.name, req_body.name.clone());
    assert_eq!(res.r#type, TagType::Plain);

    let tag_in_db = tag::Entity::find_by_id(res.id).one(&db).await?.unwrap();
    assert_eq!(tag_in_db.name, Some(req_body.name));
    assert_eq!(tag_in_db.user_id, user.id);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/tags/plain")
        .set_json(TagCreateRequest {
            name: "".to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
