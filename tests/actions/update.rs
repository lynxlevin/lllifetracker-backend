use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory;
use entities::action;
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let new_name = "action_after_update".to_string();
    let new_description = "Action after update.".to_string();
    let new_trackable = false;
    let new_color = "#ffffff".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}", action.id))
        .set_json(ActionUpdateRequest {
            name: new_name.clone(),
            description: Some(new_description.clone()),
            trackable: Some(new_trackable),
            color: Some(new_color.clone()),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ActionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, action.id);
    assert_eq!(res.name, new_name.clone());
    assert_eq!(res.description, Some(new_description.clone()));
    assert_eq!(res.trackable, new_trackable);
    assert_eq!(res.color, new_color.clone());
    assert_eq!(res.created_at, action.created_at);
    assert!(res.updated_at > action.updated_at);

    let action_in_db = action::Entity::find_by_id(action.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_in_db.user_id, user.id);
    assert_eq!(action_in_db.archived, action.archived);
    assert_eq!(ActionVisible::from(action_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}", action.id))
        .set_json(ActionUpdateRequest {
            name: "action_after_update_route".to_string(),
            description: None,
            trackable: None,
            color: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn validation_errors() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let long_name = "#1234567".to_string();
    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}", action.id))
        .set_json(ActionUpdateRequest {
            name: "action_after_update_route".to_string(),
            description: None,
            trackable: None,
            color: Some(long_name),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

    let short_name = "#12345".to_string();
    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}", action.id))
        .set_json(ActionUpdateRequest {
            name: "action_after_update_route".to_string(),
            description: None,
            trackable: None,
            color: Some(short_name),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

    let bad_format = "$ffffff".to_string();
    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}", action.id))
        .set_json(ActionUpdateRequest {
            name: "action_after_update_route".to_string(),
            description: None,
            trackable: None,
            color: Some(bad_format),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

    let bad_character = "#gggggg".to_string();
    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}", action.id))
        .set_json(ActionUpdateRequest {
            name: "action_after_update_route".to_string(),
            description: None,
            trackable: None,
            color: Some(bad_character),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

    Ok(())
}
