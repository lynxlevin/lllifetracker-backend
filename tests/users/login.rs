use actix_web::{http, test};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::users::types::LoginRequest;
use uuid::Uuid;

use crate::utils::{init_app, Connections};
use common::factory::{self, *};

#[actix_web::test]
#[ignore]
async fn happy_path() -> Result<(), DbErr> {
    unimplemented!("This is checked in integration.rs.");
}

#[actix_web::test]
async fn block_too_many_attempts_on_incorrect_password() -> Result<(), DbErr> {
    let Connections {
        app, db, settings, ..
    } = init_app().await?;
    let incorrect_password = "passworda";
    let hashed_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";
    let user = factory::user()
        .password(hashed_password)
        .insert(&db)
        .await?;

    for _ in 0..settings.application.max_login_attempts {
        let req = test::TestRequest::post()
            .uri("/api/users/login")
            .set_json(LoginRequest {
                email: user.email.to_string(),
                password: incorrect_password.to_string(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
    }

    let req = test::TestRequest::post()
        .uri("/api/users/login")
        .set_json(LoginRequest {
            email: user.email.to_string(),
            password: incorrect_password.to_string(),
        })
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod not_found {
    use super::*;

    #[actix_web::test]
    async fn incorrect_email() -> Result<(), DbErr> {
        let Connections { app, .. } = init_app().await?;
        let password = "password";

        let req = test::TestRequest::post()
            .uri("/api/users/login")
            .set_json(LoginRequest {
                email: format!("{}@test.com", Uuid::now_v7()),
                password: password.to_string(),
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn incorrect_password() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let incorrect_password = "passworda";
        let hashed_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";
        let user = factory::user()
            .password(hashed_password)
            .insert(&db)
            .await?;

        let req = test::TestRequest::post()
            .uri("/api/users/login")
            .set_json(LoginRequest {
                email: user.email.to_string(),
                password: incorrect_password.to_string(),
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }
}
