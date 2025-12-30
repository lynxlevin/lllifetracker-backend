use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn test_order() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let title_0 = "Title_0".to_string();
    let title_1 = "Title_1".to_string();
    let _reading_note_0 = factory::reading_note(user.id)
        .title(title_0.clone())
        .insert(&db)
        .await?;
    let _reading_note_1 = factory::reading_note(user.id)
        .title(title_1.clone())
        .insert(&db)
        .await?;
    let _another_reading_note_1 = factory::reading_note(user.id)
        .title(title_1.clone())
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/reading_notes/titles")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<String> = test::read_body_json(resp).await;
    let expected = vec![title_0, title_1];

    assert_eq!(body.len(), expected.len());
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/reading_notes/titles")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
