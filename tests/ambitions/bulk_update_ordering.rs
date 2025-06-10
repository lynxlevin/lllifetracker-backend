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
    let ambition_0 = factory::ambition(user.id).insert(&db).await?;
    let ambition_1 = factory::ambition(user.id).insert(&db).await?;
    let ambition_2 = factory::ambition(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri("/api/ambitions/bulk_update_ordering")
        .set_json(AmbitionBulkUpdateOrderingRequest {
            ordering: vec![ambition_0.id, ambition_1.id],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let actin_in_db_0 = ambition::Entity::find_by_id(ambition_0.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(actin_in_db_0.ordering, Some(1));

    let actin_in_db_1 = ambition::Entity::find_by_id(ambition_1.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(actin_in_db_1.ordering, Some(2));

    let ambition_in_db_2 = ambition::Entity::find_by_id(ambition_2.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(ambition_in_db_2.ordering, None);

    Ok(())
}
