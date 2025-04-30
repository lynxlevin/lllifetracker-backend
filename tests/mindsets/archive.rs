use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use test_utils::{self, *};
use types::*;
use entities::mindset;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset = factory::mindset(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/mindsets/{}/archive", mindset.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: MindsetVisible = test::read_body_json(res).await;
    assert_eq!(res.id, mindset.id);
    assert_eq!(res.name, mindset.name.clone());
    assert_eq!(res.description, mindset.description.clone());
    assert_eq!(res.created_at, mindset.created_at);
    assert!(res.updated_at > mindset.updated_at);

    let mindset_in_db = mindset::Entity::find_by_id(mindset.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(mindset_in_db.user_id, user.id);
    assert_eq!(mindset_in_db.archived, true);
    assert_eq!(MindsetVisible::from(mindset_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset = factory::mindset(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/mindsets/{}/archive", mindset.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
