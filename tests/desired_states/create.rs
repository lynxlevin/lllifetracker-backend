use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use use_cases::my_way::desired_states::types::{DesiredStateCreateRequest, DesiredStateVisible};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;
use entities::{desired_state, sea_orm_active_enums::TagType, tag};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let name = "create_desired_state route happy path".to_string();
    let description = "Create desired_state route happy path.".to_string();
    let category = factory::desired_state_category(user.id).insert(&db).await?;

    let req = test::TestRequest::post()
        .uri("/api/desired_states")
        .set_json(DesiredStateCreateRequest {
            name: name.clone(),
            description: Some(description.clone()),
            category_id: Some(category.id),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: DesiredStateVisible = test::read_body_json(res).await;
    assert_eq!(res.name, name);
    assert_eq!(res.description, Some(description));
    assert_eq!(res.category_id, Some(category.id));

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
        .filter(tag::Column::Type.eq(TagType::DesiredState))
        .one(&db)
        .await?;
    assert!(tag_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn no_category_cases() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let other_user_category = factory::desired_state_category(other_user.id)
        .insert(&db)
        .await?;

    for (category_id, case) in vec![
        (other_user_category.id, "other_user_category.id"),
        (Uuid::now_v7(), "non-existent-category-id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::post()
            .uri("/api/desired_states")
            .set_json(DesiredStateCreateRequest {
                name: String::default(),
                description: None,
                category_id: Some(category_id),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let res: DesiredStateVisible = test::read_body_json(res).await;
        assert_eq!(res.category_id, None);

        let desired_state_in_db = desired_state::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db.user_id, user.id);
        assert_eq!(DesiredStateVisible::from(desired_state_in_db), res);
    }

    Ok(())
}
