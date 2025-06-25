use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::desired_states::types::{DesiredStateUpdateRequest, DesiredStateVisible};
use uuid::Uuid;

use super::super::utils::init_app;
use common::factory;
use entities::desired_state;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state = factory::desired_state(user.id).insert(&db).await?;
    let category = factory::desired_state_category(user.id).insert(&db).await?;

    let new_name = "desired_state_after_update".to_string();
    let new_description = "DesiredState after update.".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/desired_states/{}", desired_state.id))
        .set_json(DesiredStateUpdateRequest {
            name: new_name.clone(),
            description: Some(new_description.clone()),
            category_id: Some(category.id),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: DesiredStateVisible = test::read_body_json(res).await;
    assert_eq!(res.id, desired_state.id);
    assert_eq!(res.name, new_name.clone());
    assert_eq!(res.description, Some(new_description.clone()));
    assert_eq!(res.category_id, Some(category.id));
    assert_eq!(res.created_at, desired_state.created_at);
    assert!(res.updated_at > desired_state.updated_at);

    let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(desired_state_in_db.user_id, user.id);
    assert_eq!(desired_state_in_db.archived, desired_state.archived);
    assert_eq!(DesiredStateVisible::from(desired_state_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn no_category_cases() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let desired_state = factory::desired_state(user.id).insert(&db).await?;
    let other_user_category = factory::desired_state_category(other_user.id)
        .insert(&db)
        .await?;

    for (category_id, case) in vec![
        (other_user_category.id, "other_user_category.id"),
        (Uuid::now_v7(), "non-existent-category-id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/desired_states/{}", desired_state.id))
            .set_json(DesiredStateUpdateRequest {
                name: String::default(),
                description: None,
                category_id: Some(category_id),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let res: DesiredStateVisible = test::read_body_json(res).await;
        assert_eq!(res.id, desired_state.id);
        assert_eq!(res.category_id, None);

        let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db.user_id, user.id);
        assert_eq!(DesiredStateVisible::from(desired_state_in_db), res);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state = factory::desired_state(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/desired_states/{}", desired_state.id))
        .set_json(DesiredStateUpdateRequest {
            name: "desired_state".to_string(),
            description: None,
            category_id: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
