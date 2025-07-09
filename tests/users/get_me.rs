use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::users::types::UserVisible;

use crate::utils::{init_app, Connections};
use common::factory;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req = test::TestRequest::get().uri("/api/users/me").to_request();
    req.extensions_mut().insert(user.clone());
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    let res: UserVisible = test::read_body_json(res).await;
    assert_eq!(res.id, user.id);
    assert_eq!(res.first_name, user.first_name);
    assert_eq!(res.last_name, user.last_name);
    assert_eq!(res.email, user.email);
    assert_eq!(res.is_active, user.is_active);
    assert_eq!(res.first_track_at, user.first_track_at);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri("/api/users/me").to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
