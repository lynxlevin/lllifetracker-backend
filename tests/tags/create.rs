use actix_web::{http, test, HttpMessage};
use entities::tag;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use test_utils::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req_body = TagCreateRequest {
        name: "new_tag".to_string(),
    };

    let req = test::TestRequest::post()
        .set_json(req_body.clone())
        .uri("/api/tags")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::CREATED);

    let res: TagVisible = test::read_body_json(resp).await;
    assert_eq!(res.name, req_body.name.clone());
    assert_eq!(res.tag_type, TagType::Plain);

    let tag_in_db = tag::Entity::find_by_id(res.id).one(&db).await?.unwrap();
    assert_eq!(tag_in_db.name, Some(req_body.name));
    assert_eq!(tag_in_db.user_id, user.id);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get().uri("/api/tags").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
