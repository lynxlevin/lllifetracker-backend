use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use types::ActionBulkUpdateOrderRequest;

use super::super::utils::init_app;
use common::factory;
use entities::action;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action_0 = factory::action(user.id).insert(&db).await?;
    let action_1 = factory::action(user.id).insert(&db).await?;
    let action_2 = factory::action(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri("/api/actions/bulk_update_ordering")
        .set_json(ActionBulkUpdateOrderRequest {
            ordering: vec![action_0.id, action_1.id],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let actin_in_db_0 = action::Entity::find_by_id(action_0.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(actin_in_db_0.ordering, Some(1));

    let actin_in_db_1 = action::Entity::find_by_id(action_1.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(actin_in_db_1.ordering, Some(2));

    let action_in_db_2 = action::Entity::find_by_id(action_2.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_in_db_2.ordering, None);

    Ok(())
}
