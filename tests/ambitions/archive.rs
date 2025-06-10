use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory;
use entities::ambition;
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let ambition = factory::ambition(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/ambitions/{}/archive", ambition.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: AmbitionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, ambition.id);
    assert_eq!(res.name, ambition.name.clone());
    assert_eq!(res.description, ambition.description.clone());
    assert_eq!(res.created_at, ambition.created_at);
    assert!(res.updated_at > ambition.updated_at);

    let ambition_in_db = ambition::Entity::find_by_id(ambition.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(ambition_in_db.user_id, user.id);
    assert_eq!(ambition_in_db.archived, true);
    assert_eq!(AmbitionVisible::from(ambition_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let ambition = factory::ambition(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/ambitions/{}/archive", ambition.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
