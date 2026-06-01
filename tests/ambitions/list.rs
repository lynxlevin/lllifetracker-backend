use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::ambitions::types::AmbitionVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let ambitions = create_ambitions(
        vec![
            AmbitionParam { name: "ambition_0".to_string(), archived: false, ..Default::default() },
            AmbitionParam { name: "ambition_1".to_string(), archived: false, ..Default::default() },
            AmbitionParam { name: "archived_ambition".to_string(), archived: true, ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let ambition_0 = ambitions.get("ambition_0").unwrap();
    let ambition_1 = ambitions.get("ambition_1").unwrap();
    let archived_ambition = ambitions.get("archived_ambition").unwrap();

    let req = test::TestRequest::get().uri("/api/ambitions").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<AmbitionVisible> = test::read_body_json(resp).await;
    let expected = vec![
        AmbitionVisible::from(ambition_0),
        AmbitionVisible::from(ambition_1),
        AmbitionVisible::from(archived_ambition),
    ];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);
    assert_eq!(res[1], expected[1]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri("/api/ambitions").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
