use actix_web::{http, test, HttpMessage};
use sea_orm::{entity::prelude::ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use test_utils::{self, *};
use types::*;
use entities::desired_state;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state_0 = factory::desired_state(user.id).insert(&db).await?;
    let desired_state_1 = factory::desired_state(user.id).insert(&db).await?;
    let desired_state_2 = factory::desired_state(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri("/api/desired_states/bulk_update_ordering")
        .set_json(DesiredStateBulkUpdateOrderingRequest {
            ordering: vec![desired_state_0.id, desired_state_1.id],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let actin_in_db_0 = desired_state::Entity::find_by_id(desired_state_0.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(actin_in_db_0.ordering, Some(1));

    let actin_in_db_1 = desired_state::Entity::find_by_id(desired_state_1.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(actin_in_db_1.ordering, Some(2));

    let desired_state_in_db_2 = desired_state::Entity::find_by_id(desired_state_2.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(desired_state_in_db_2.ordering, None);

    Ok(())
}
