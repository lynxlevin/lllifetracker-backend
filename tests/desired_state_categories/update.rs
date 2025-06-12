use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use uuid::Uuid;

use super::super::utils::init_app;
use common::factory;
use entities::desired_state_category;
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let category = factory::desired_state_category(user.id).insert(&db).await?;

    let new_name = "new name".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/desired_state_categories/{}", category.id))
        .set_json(DesiredStateCategoryUpdateRequest {
            name: new_name.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: DesiredStateCategoryVisible = test::read_body_json(res).await;
    assert_eq!(res.id, category.id);
    assert_eq!(res.name, new_name);
    assert_eq!(res.ordering, category.ordering);

    let category_in_db = desired_state_category::Entity::find_by_id(category.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(category_in_db.user_id, user.id);
    assert_eq!(DesiredStateCategoryVisible::from(category_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn not_found_cases() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let other_user_category = factory::desired_state_category(other_user.id)
        .insert(&db)
        .await?;

    for (category_id, case) in vec![
        (other_user_category.id, "other_user_category.id"),
        (Uuid::now_v7(), "non-existent-id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/desired_state_categories/{}", category_id))
            .set_json(DesiredStateCategoryUpdateRequest {
                name: String::default(),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/desired_state_categories/{}", Uuid::now_v7()))
        .set_json(DesiredStateCategoryUpdateRequest {
            name: String::default(),
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
