use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::directions::types::{DirectionUpdateRequest, DirectionVisible};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;
use entities::direction;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let direction = factory::direction(user.id).insert(&db).await?;
    let category = factory::direction_category(user.id).insert(&db).await?;

    let new_name = "direction_after_update".to_string();
    let new_description = "Direction after update.".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/directions/{}", direction.id))
        .set_json(DirectionUpdateRequest {
            name: new_name.clone(),
            description: Some(new_description.clone()),
            category_id: Some(category.id),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: DirectionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, direction.id);
    assert_eq!(res.name, new_name.clone());
    assert_eq!(res.description, Some(new_description.clone()));
    assert_eq!(res.category_id, Some(category.id));
    assert_eq!(res.created_at, direction.created_at);
    assert!(res.updated_at > direction.updated_at);

    let direction_in_db = direction::Entity::find_by_id(direction.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(direction_in_db.user_id, user.id);
    assert_eq!(direction_in_db.archived, direction.archived);
    assert_eq!(DirectionVisible::from(direction_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn no_category_cases() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let direction = factory::direction(user.id).insert(&db).await?;
    let other_user_category = factory::direction_category(other_user.id)
        .insert(&db)
        .await?;

    for (category_id, case) in vec![
        (other_user_category.id, "other_user_category.id"),
        (Uuid::now_v7(), "non-existent-category-id"),
    ] {
        dbg!(case);
        let req = test::TestRequest::put()
            .uri(&format!("/api/directions/{}", direction.id))
            .set_json(DirectionUpdateRequest {
                name: String::default(),
                description: None,
                category_id: Some(category_id),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let res: DirectionVisible = test::read_body_json(res).await;
        assert_eq!(res.id, direction.id);
        assert_eq!(res.category_id, None);

        let direction_in_db = direction::Entity::find_by_id(direction.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(direction_in_db.user_id, user.id);
        assert_eq!(DirectionVisible::from(direction_in_db), res);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let direction = factory::direction(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/directions/{}", direction.id))
        .set_json(DirectionUpdateRequest {
            name: "direction".to_string(),
            description: None,
            category_id: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
