use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use entities::{desired_state, mindset, tag};
use test_utils::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (desired_state, desired_state_tag) =
        factory::desired_state(user.id).insert_with_tag(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/desired_states/{}/convert", desired_state.id))
        .set_json(DesiredStateConvertRequest {
            convert_to: DesiredStateConvertToType::Mindset,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: MindsetVisible = test::read_body_json(res).await;
    assert_eq!(res.name, desired_state.name.clone());
    assert_eq!(res.description, desired_state.description);
    assert_ne!(res.created_at, desired_state.created_at);
    assert_ne!(res.updated_at, desired_state.updated_at);

    let mindset_in_db = mindset::Entity::find_by_id(res.id).one(&db).await?.unwrap();
    assert_eq!(MindsetVisible::from(mindset_in_db), res);

    let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
        .one(&db)
        .await?;
    assert!(desired_state_in_db.is_none());

    let tag_in_db = tag::Entity::find_by_id(desired_state_tag.id)
        .one(&db)
        .await?
        .unwrap();
    assert!(tag_in_db.ambition_id.is_none());
    assert!(tag_in_db.desired_state_id.is_none());
    assert_eq!(tag_in_db.mindset_id, Some(res.id));
    assert!(tag_in_db.action_id.is_none());
    assert!(tag_in_db.name.is_none());
    assert_eq!(tag_in_db.user_id, user.id);
    assert_eq!(tag_in_db.created_at, desired_state_tag.created_at);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state = factory::desired_state(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/desired_states/{}/convert", desired_state.id))
        .set_json(DesiredStateConvertRequest {
            convert_to: DesiredStateConvertToType::Mindset,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
