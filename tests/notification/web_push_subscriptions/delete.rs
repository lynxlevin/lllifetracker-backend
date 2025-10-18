use actix_web::{http, test, HttpMessage};
use entities::web_push_subscription;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use crate::utils::{init_app, Connections};
use common::factory;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, settings } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _subscription = factory::web_push_subscription(user.id, &settings)
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri("/api/web_push_subscription")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let sub_in_db = web_push_subscription::Entity::find()
        .filter(web_push_subscription::Column::UserId.eq(user.id))
        .one(&db)
        .await?;
    assert_eq!(sub_in_db, None);

    Ok(())
}

#[actix_web::test]
async fn happy_path_no_subscription() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri("/api/web_push_subscription")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::delete()
        .uri("/api/web_push_subscription")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
