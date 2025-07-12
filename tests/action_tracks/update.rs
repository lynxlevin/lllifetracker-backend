use actix_web::{http, test, HttpMessage};
use chrono::{DateTime, SubsecRound, TimeDelta, Utc};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::action_tracks::types::{ActionTrackUpdateRequest, ActionTrackVisible};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{action_track, user};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .insert(&db)
        .await?;
    let ended_at = DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap();
    let duration = 180;
    let started_at = ended_at - chrono::TimeDelta::seconds(duration.into());

    let req = test::TestRequest::put()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .set_json(ActionTrackUpdateRequest {
            action_id: action.id,
            started_at,
            ended_at: Some(ended_at),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ActionTrackVisible = test::read_body_json(res).await;
    assert_eq!(res.id, action_track.id);
    assert_eq!(res.action_id, action.id);
    assert_eq!(res.started_at, started_at.trunc_subsecs(0));
    assert_eq!(res.ended_at, Some(ended_at.trunc_subsecs(0)));
    assert_eq!(res.duration, Some(duration));

    let action_track_in_db = action_track::Entity::find_by_id(action_track.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_track_in_db.user_id, user.id);
    assert_eq!(ActionTrackVisible::from(action_track_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn conflict_on_duplicate_starts_at() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .insert(&db)
        .await?;
    let existing_action_track = factory::action_track(user.id)
        .started_at(action_track.started_at + TimeDelta::seconds(1))
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .set_json(ActionTrackUpdateRequest {
            action_id: action.id,
            started_at: existing_action_track.started_at,
            ended_at: None,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CONFLICT);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .set_json(ActionTrackUpdateRequest {
            action_id: uuid::Uuid::now_v7(),
            started_at: Utc::now().into(),
            ended_at: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[cfg(test)]
mod user_first_track_at_update {
    use super::*;

    #[actix_web::test]
    async fn update_for_user_with_no_first_starts_at_sets_it() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;
        let started_at = Utc::now().into();

        let req = test::TestRequest::put()
            .uri(&format!("/api/action_tracks/{}", action_track.id))
            .set_json(ActionTrackUpdateRequest {
                action_id: action.id,
                started_at,
                ended_at: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
        assert_eq!(user_in_db.first_track_at, Some(started_at.trunc_subsecs(0)));

        Ok(())
    }

    #[actix_web::test]
    async fn newer_started_at_than_first_track_at_makes_no_change() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let original_first_track_at =
            Some(DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap());
        let user = factory::user()
            .first_track_at(original_first_track_at)
            .insert(&db)
            .await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;
        let started_at = Utc::now().into();

        let req = test::TestRequest::put()
            .uri(&format!("/api/action_tracks/{}", action_track.id))
            .set_json(ActionTrackUpdateRequest {
                action_id: action.id,
                started_at,
                ended_at: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
        assert_eq!(user_in_db.first_track_at, original_first_track_at);

        Ok(())
    }

    #[actix_web::test]
    async fn older_started_at_than_first_track_at_updates_user() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user()
            .first_track_at(Some(
                DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap(),
            ))
            .insert(&db)
            .await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;

        let started_at = DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/api/action_tracks/{}", action_track.id))
            .set_json(ActionTrackUpdateRequest {
                action_id: action.id,
                started_at,
                ended_at: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
        assert_eq!(user_in_db.first_track_at, Some(started_at));

        Ok(())
    }

    #[actix_web::test]
    async fn updating_oldest_track_switches_to_next_oldest() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user()
            .first_track_at(Some(
                DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap(),
            ))
            .insert(&db)
            .await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .started_at(DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap())
            .insert(&db)
            .await?;
        let second_oldest_action_track = factory::action_track(user.id)
            .action_id(action.id)
            .started_at(DateTime::parse_from_rfc3339("2025-07-09T00:00:00Z").unwrap())
            .insert(&db)
            .await?;

        let started_at = DateTime::parse_from_rfc3339("2025-07-10T00:00:00Z").unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/api/action_tracks/{}", action_track.id))
            .set_json(ActionTrackUpdateRequest {
                action_id: action.id,
                started_at,
                ended_at: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
        assert_eq!(
            user_in_db.first_track_at,
            Some(second_oldest_action_track.started_at)
        );

        Ok(())
    }

    #[actix_web::test]
    async fn updating_only_track_updates_first_track_at_to_same() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user()
            .first_track_at(Some(
                DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap(),
            ))
            .insert(&db)
            .await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .started_at(DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap())
            .insert(&db)
            .await?;

        let started_at = DateTime::parse_from_rfc3339("2025-07-10T00:00:00Z").unwrap();

        let req = test::TestRequest::put()
            .uri(&format!("/api/action_tracks/{}", action_track.id))
            .set_json(ActionTrackUpdateRequest {
                action_id: action.id,
                started_at,
                ended_at: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
        assert_eq!(user_in_db.first_track_at, Some(started_at));

        Ok(())
    }
}
