use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory;
use entities::mindset;
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset_0 = factory::mindset(user.id).insert(&db).await?;
    let mindset_1 = factory::mindset(user.id).insert(&db).await?;
    let mindset_2 = factory::mindset(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri("/api/mindsets/bulk_update_ordering")
        .set_json(MindsetBulkUpdateOrderingRequest {
            ordering: vec![mindset_0.id, mindset_1.id],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let actin_in_db_0 = mindset::Entity::find_by_id(mindset_0.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(actin_in_db_0.ordering, Some(1));

    let actin_in_db_1 = mindset::Entity::find_by_id(mindset_1.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(actin_in_db_1.ordering, Some(2));

    let mindset_in_db_2 = mindset::Entity::find_by_id(mindset_2.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(mindset_in_db_2.ordering, None);

    Ok(())
}
