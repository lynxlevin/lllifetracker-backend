use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::notification::web_push_subscription::types::WebPushSubscriptionVisible;

use crate::utils::{init_app, Connections};
use common::factory;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, settings } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let subscription = factory::web_push_subscription(user.id, &settings)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/web_push_subscription")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Option<WebPushSubscriptionVisible> = test::read_body_json(resp).await;
    let expected = Some(WebPushSubscriptionVisible {
        device_name: subscription.device_name.clone(),
        expiration_epoch_time: subscription.expiration_epoch_time,
    });

    assert_eq!(body, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/web_push_subscription")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
