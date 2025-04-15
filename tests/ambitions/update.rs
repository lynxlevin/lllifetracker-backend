use actix_web::{http, test, HttpMessage};
use sea_orm::{entity::prelude::ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use entities::ambition;
use test_utils::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let ambition = factory::ambition(user.id).insert(&db).await?;

    let new_name = "ambition_after_update_route".to_string();
    let new_description = Some("edited_description".to_string());

    let req = test::TestRequest::put()
        .uri(&format!("/api/ambitions/{}", ambition.id))
        .set_json(AmbitionUpdateRequest {
            name: new_name.clone(),
            description: new_description.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: AmbitionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, ambition.id);
    assert_eq!(res.name, new_name.clone());
    assert_eq!(res.description, new_description.clone());
    assert_eq!(res.created_at, ambition.created_at);
    assert!(res.updated_at > ambition.updated_at);

    let ambition_in_db = ambition::Entity::find_by_id(ambition.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(ambition_in_db.user_id, user.id);
    assert_eq!(AmbitionVisible::from(ambition_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn happy_path_no_description() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let ambition = factory::ambition(user.id)
        .description(Some("Original description".to_string()))
        .insert(&db)
        .await?;

    let new_name = "ambition_after_update_route".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/ambitions/{}", ambition.id))
        .set_json(AmbitionUpdateRequest {
            name: new_name.clone(),
            description: None,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: AmbitionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, ambition.id);
    assert_eq!(res.name, new_name.clone());
    assert!(res.description.is_none());
    assert_eq!(res.created_at, ambition.created_at);
    assert!(res.updated_at > ambition.updated_at);

    let ambition_in_db = ambition::Entity::find_by_id(ambition.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(ambition_in_db.user_id, user.id);
    assert_eq!(ambition_in_db.archived, ambition.archived);
    assert_eq!(AmbitionVisible::from(ambition_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let ambition = factory::ambition(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/ambitions/{}", ambition.id))
        .set_json(AmbitionUpdateRequest {
            name: "ambition_after_update_route".to_string(),
            description: Some("edited_description".to_string()),
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
