use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory;
use entities::desired_state;
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state = factory::desired_state(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/desired_states/{}/archive", desired_state.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: DesiredStateVisible = test::read_body_json(res).await;
    assert_eq!(res.id, desired_state.id);
    assert_eq!(res.name, desired_state.name.clone());
    assert_eq!(res.description, desired_state.description.clone());
    assert_eq!(res.created_at, desired_state.created_at);
    assert!(res.updated_at > desired_state.updated_at);

    let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(desired_state_in_db.user_id, user.id);
    assert_eq!(desired_state_in_db.archived, true);
    assert_eq!(DesiredStateVisible::from(desired_state_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state = factory::desired_state(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/desired_states/{}/archive", desired_state.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
