use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use entities::mindset;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset = factory::mindset(user.id).insert(&db).await?;

    let new_name = "mindset_after_update_route".to_string();
    let new_description = Some("edited_description".to_string());

    let req = test::TestRequest::put()
        .uri(&format!("/api/mindsets/{}", mindset.id))
        .set_json(MindsetUpdateRequest {
            name: new_name.clone(),
            description: new_description.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: MindsetVisible = test::read_body_json(res).await;
    assert_eq!(res.id, mindset.id);
    assert_eq!(res.name, new_name.clone());
    assert_eq!(res.description, new_description.clone());
    assert_eq!(res.created_at, mindset.created_at);
    assert!(res.updated_at > mindset.updated_at);

    let mindset_in_db = mindset::Entity::find_by_id(mindset.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(mindset_in_db.user_id, user.id);
    assert_eq!(MindsetVisible::from(mindset_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn happy_path_no_description() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset = factory::mindset(user.id)
        .description(Some("Original description".to_string()))
        .insert(&db)
        .await?;

    let new_name = "mindset_after_update_route".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/mindsets/{}", mindset.id))
        .set_json(MindsetUpdateRequest {
            name: new_name.clone(),
            description: None,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: MindsetVisible = test::read_body_json(res).await;
    assert_eq!(res.id, mindset.id);
    assert_eq!(res.name, new_name.clone());
    assert!(res.description.is_none());
    assert_eq!(res.created_at, mindset.created_at);
    assert!(res.updated_at > mindset.updated_at);

    let mindset_in_db = mindset::Entity::find_by_id(mindset.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(mindset_in_db.user_id, user.id);
    assert_eq!(mindset_in_db.archived, mindset.archived);
    assert_eq!(MindsetVisible::from(mindset_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset = factory::mindset(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/mindsets/{}", mindset.id))
        .set_json(MindsetUpdateRequest {
            name: "mindset_after_update_route".to_string(),
            description: Some("edited_description".to_string()),
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
