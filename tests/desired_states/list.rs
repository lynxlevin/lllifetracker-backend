use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::desired_states::types::DesiredStateVisible;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state_0 = factory::desired_state(user.id)
        .name("desired_state_0".to_string())
        .insert(&db)
        .await?;
    let desired_state_1 = factory::desired_state(user.id)
        .name("desired_state_1".to_string())
        .insert(&db)
        .await?;
    let _archived_desired_state = factory::desired_state(user.id)
        .archived(true)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/desired_states")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<DesiredStateVisible> = test::read_body_json(resp).await;
    let expected = vec![
        DesiredStateVisible::from(desired_state_0),
        DesiredStateVisible::from(desired_state_1),
    ];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);
    assert_eq!(res[1], expected[1]);

    Ok(())
}

#[actix_web::test]
async fn happy_path_show_archived_only() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _desired_state = factory::desired_state(user.id).insert(&db).await?;
    let archived_desired_state = factory::desired_state(user.id)
        .archived(true)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/desired_states?show_archived_only=true")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<DesiredStateVisible> = test::read_body_json(resp).await;
    let expected = vec![DesiredStateVisible::from(archived_desired_state)];

    assert_eq!(body.len(), expected.len());
    assert_eq!(body[0], expected[0]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/desired_states")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
