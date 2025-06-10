use actix_web::{http, test, HttpMessage};
use entities::tag;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let tag = factory::tag(user.id).insert(&db).await?;

    let req_body = TagUpdateRequest {
        name: "new_name".to_string(),
    };

    let req = test::TestRequest::put()
        .set_json(req_body.clone())
        .uri(&format!("/api/tags/plain/{}", tag.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: TagVisible = test::read_body_json(resp).await;
    assert_eq!(res.name, req_body.name.clone());
    assert_eq!(res.tag_type, TagType::Plain);

    let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?.unwrap();
    assert_eq!(tag_in_db.name, Some(req_body.name));
    assert_eq!(tag_in_db.user_id, user.id);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let tag = factory::tag(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tags/plain/{}", tag.id))
        .set_json(TagUpdateRequest {
            name: "".to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn not_found_if_non_existent_tag_id() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tags/plain/{}", uuid::Uuid::now_v7()))
        .set_json(TagUpdateRequest {
            name: "".to_string(),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn bad_request_if_not_plain_tag() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/tags/plain/{}", ambition_tag.id))
        .set_json(TagUpdateRequest {
            name: "".to_string(),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

    Ok(())
}
