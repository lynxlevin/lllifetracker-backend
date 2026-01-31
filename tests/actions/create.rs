use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use use_cases::my_way::actions::types::{ActionCreateRequest, ActionVisible};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;
use entities::{
    action,
    sea_orm_active_enums::{ActionTrackType, TagType},
    tag,
};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let name = "create_action".to_string();
    let discipline = "Create action.".to_string();
    let memo = "Something important.".to_string();
    let req = test::TestRequest::post()
        .uri("/api/actions")
        .set_json(ActionCreateRequest {
            name: name.clone(),
            discipline: Some(discipline.clone()),
            memo: Some(memo.clone()),
            track_type: ActionTrackType::Count,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: ActionVisible = test::read_body_json(res).await;
    assert_eq!(res.name, name.clone());
    assert_eq!(res.discipline, Some(discipline.clone()));
    assert_eq!(res.memo, Some(memo.clone()));
    assert_eq!(res.track_type, ActionTrackType::Count);

    let action_in_db = action::Entity::find_by_id(res.id).one(&db).await?.unwrap();
    assert_eq!(action_in_db.user_id, user.id);
    assert_eq!(ActionVisible::from(action_in_db), res);

    let tag_in_db = tag::Entity::find()
        .filter(tag::Column::UserId.eq(user.id))
        .filter(tag::Column::ActionId.eq(res.id))
        .filter(tag::Column::AmbitionId.is_null())
        .filter(tag::Column::DesiredStateId.is_null())
        .filter(tag::Column::Type.eq(TagType::Action))
        .one(&db)
        .await?;
    assert!(tag_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/actions")
        .set_json(ActionCreateRequest {
            name: "Test create_action not logged in".to_string(),
            discipline: None,
            memo: None,
            track_type: ActionTrackType::TimeSpan,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
