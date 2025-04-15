use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use super::super::utils::init_app;
use test_utils::{self, *};
use types::*;
use entities::{desired_state, tag};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let name = "create_desired_state route happy path".to_string();
    let description = "Create desired_state route happy path.".to_string();

    let req = test::TestRequest::post()
        .uri("/api/desired_states")
        .set_json(DesiredStateCreateRequest {
            name: name.clone(),
            description: Some(description.clone()),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: DesiredStateVisible = test::read_body_json(res).await;
    assert_eq!(res.name, name);

    let desired_state_in_db = desired_state::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(desired_state_in_db.user_id, user.id);
    assert_eq!(DesiredStateVisible::from(desired_state_in_db), res);

    let tag_in_db = tag::Entity::find()
        .filter(tag::Column::AmbitionId.is_null())
        .filter(tag::Column::DesiredStateId.eq(res.id))
        .filter(tag::Column::ActionId.is_null())
        .filter(tag::Column::UserId.eq(user.id))
        .one(&db)
        .await?;
    assert!(tag_in_db.is_some());

    Ok(())
}
