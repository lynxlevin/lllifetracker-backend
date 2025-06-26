use actix_web::{cookie::Cookie, http, test};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::users::types::LoginRequest;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
async fn login_to_get_me_to_logout() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let password = "password";
    let hashed_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";
    let user = factory::user()
        .password(hashed_password)
        .insert(&db)
        .await?;

    let login_req = test::TestRequest::post()
        .uri("/api/users/login")
        .set_json(LoginRequest {
            email: user.email.to_string(),
            password: password.to_string(),
        })
        .to_request();
    let res = test::call_service(&app, login_req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let headers = res.headers();
    let mut set_cookie_header = headers.get_all("set-cookie");
    let session_set_cookie =
        set_cookie_header.find(|sc| sc.to_str().unwrap().starts_with("sessionId="));
    assert!(session_set_cookie.is_some());
    let session_cookie =
        Cookie::parse(urlencoding::decode(session_set_cookie.unwrap().to_str().unwrap()).unwrap())
            .unwrap();

    let check_req = test::TestRequest::get()
        .uri("/api/users/me")
        .cookie(session_cookie.clone())
        .to_request();
    let res = test::call_service(&app, check_req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let logout_req = test::TestRequest::post()
        .uri("/api/users/logout")
        .cookie(session_cookie.clone())
        .to_request();
    let res = test::call_service(&app, logout_req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let check_req = test::TestRequest::get()
        .uri("/api/users/me")
        .cookie(session_cookie.clone())
        .to_request();
    let res = test::call_service(&app, check_req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
