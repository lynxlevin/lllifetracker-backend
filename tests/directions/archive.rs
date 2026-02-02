use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::directions::types::DirectionVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;
use entities::direction;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let direction = factory::direction(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/directions/{}/archive", direction.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: DirectionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, direction.id);
    assert_eq!(res.name, direction.name.clone());
    assert_eq!(res.description, direction.description.clone());
    assert_eq!(res.created_at, direction.created_at);
    assert!(res.updated_at > direction.updated_at);

    let direction_in_db = direction::Entity::find_by_id(direction.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(direction_in_db.user_id, user.id);
    assert_eq!(direction_in_db.archived, true);
    assert_eq!(DirectionVisible::from(direction_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let direction = factory::direction(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/directions/{}/archive", direction.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
