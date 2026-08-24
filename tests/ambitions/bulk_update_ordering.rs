use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::ambitions::types::AmbitionBulkUpdateOrderingRequest;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::ambition;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    let ambitions = create_ambitions(
        vec![
            AmbitionParam { name: "ambition_0", ..Default::default() },
            AmbitionParam { name: "ambition_1", ..Default::default() },
            AmbitionParam { name: "ambition_2", ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let ambition_0 = ambitions.get("ambition_0").unwrap();
    let ambition_1 = ambitions.get("ambition_1").unwrap();
    let ambition_2 = ambitions.get("ambition_2").unwrap();

    let req = test::TestRequest::put()
        .uri("/api/ambitions/bulk_update_ordering")
        .set_json(AmbitionBulkUpdateOrderingRequest { ordering: vec![ambition_0.id, ambition_1.id] })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let actin_in_db_0 = ambition::Entity::find_by_id(ambition_0.id).one(&db.db).await?.unwrap();
    assert_eq!(actin_in_db_0.ordering, Some(1));

    let actin_in_db_1 = ambition::Entity::find_by_id(ambition_1.id).one(&db.db).await?.unwrap();
    assert_eq!(actin_in_db_1.ordering, Some(2));

    let ambition_in_db_2 = ambition::Entity::find_by_id(ambition_2.id).one(&db.db).await?.unwrap();
    assert_eq!(ambition_in_db_2.ordering, None);

    Ok(())
}
