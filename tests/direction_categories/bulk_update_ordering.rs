use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::direction_categories::types::DirectionCategoryBulkUpdateOrderingRequest;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;
use entities::direction_category;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let category_0 = factory::direction_category(user.id).insert(&db).await?;
    let category_1 = factory::direction_category(user.id).insert(&db).await?;
    let category_2 = factory::direction_category(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri("/api/direction_categories/bulk_update_ordering")
        .set_json(DirectionCategoryBulkUpdateOrderingRequest {
            ordering: vec![category_0.id, category_1.id],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let category_in_db_0 = direction_category::Entity::find_by_id(category_0.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(category_in_db_0.ordering, Some(1));

    let category_in_db_1 = direction_category::Entity::find_by_id(category_1.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(category_in_db_1.ordering, Some(2));

    let category_in_db_2 = direction_category::Entity::find_by_id(category_2.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(category_in_db_2.ordering, None);

    Ok(())
}
