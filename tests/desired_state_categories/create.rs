use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::desired_state_categories::types::{
    DesiredStateCategoryCreateRequest, DesiredStateCategoryVisible,
};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::desired_state_category;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _other_category = factory::desired_state_category(user.id)
        .ordering(Some(1))
        .insert(&db)
        .await?;
    let _null_ordering_category = factory::desired_state_category(user.id).insert(&db).await?;
    let name = "new_category".to_string();

    let req = test::TestRequest::post()
        .uri("/api/desired_state_categories")
        .set_json(DesiredStateCategoryCreateRequest { name: name.clone() })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: DesiredStateCategoryVisible = test::read_body_json(res).await;
    assert_eq!(res.name, name);

    let category_in_db = desired_state_category::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(category_in_db.user_id, user.id);
    assert_eq!(category_in_db.ordering, Some(3));
    assert_eq!(DesiredStateCategoryVisible::from(category_in_db), res);

    Ok(())
}
