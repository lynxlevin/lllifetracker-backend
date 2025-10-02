use actix_web::{http, test, HttpMessage};
use entities::web_push_subscription;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use use_cases::notification::web_push_subscription::types::{
    WebPushSubscriptionCreateRequest, WebPushSubscriptionVisible,
};

use crate::utils::{init_app, Connections};
use common::{
    db::{decode_and_decrypt, encrypt_and_encode},
    factory,
};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, settings } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req_body = WebPushSubscriptionCreateRequest {
        device_name: "My iPhone".to_string(),
        endpoint: "https://sample.push.com".to_string(),
        expiration_epoch_time: Some(1759125917),
        p256dh_key: "p256key".to_string(),
        auth_key: "auth_key".to_string(),
    };

    let req = test::TestRequest::post()
        .set_json(req_body.clone())
        .uri("/api/web_push_subscription")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::CREATED);

    let res: WebPushSubscriptionVisible = test::read_body_json(resp).await;
    assert_eq!(res.device_name, req_body.device_name.clone());
    assert_eq!(res.expiration_epoch_time, req_body.expiration_epoch_time);

    let sub_in_db = web_push_subscription::Entity::find()
        .filter(web_push_subscription::Column::UserId.eq(user.id))
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(sub_in_db.device_name, req_body.device_name);
    assert_eq!(
        sub_in_db.expiration_epoch_time,
        req_body.expiration_epoch_time
    );
    assert_eq!(
        sub_in_db.endpoint,
        encrypt_and_encode(req_body.endpoint.clone(), &settings).unwrap()
    );
    assert_eq!(
        sub_in_db.p256dh_key,
        encrypt_and_encode(req_body.p256dh_key.clone(), &settings).unwrap()
    );
    assert_eq!(
        sub_in_db.auth_key,
        encrypt_and_encode(req_body.auth_key.clone(), &settings).unwrap()
    );
    assert_eq!(
        decode_and_decrypt(sub_in_db.endpoint, &settings).unwrap(),
        req_body.endpoint,
    );
    assert_eq!(
        decode_and_decrypt(sub_in_db.p256dh_key, &settings).unwrap(),
        req_body.p256dh_key,
    );
    assert_eq!(
        decode_and_decrypt(sub_in_db.auth_key, &settings).unwrap(),
        req_body.auth_key,
    );

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/web_push_subscription")
        .set_json(WebPushSubscriptionCreateRequest {
            ..Default::default()
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
