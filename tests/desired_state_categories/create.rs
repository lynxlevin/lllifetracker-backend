use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory;
use entities::desired_state_category;
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
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
    assert_eq!(res.ordering, None);

    let desired_state_category_in_db = desired_state_category::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(desired_state_category_in_db.user_id, user.id);
    assert_eq!(
        DesiredStateCategoryVisible::from(desired_state_category_in_db),
        res
    );

    Ok(())
}
