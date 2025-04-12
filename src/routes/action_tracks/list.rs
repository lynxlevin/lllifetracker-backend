use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use serde::Deserialize;
use services::action_track_query::ActionTrackQuery;

#[derive(Deserialize, Debug)]
struct QueryParam {
    active_only: Option<bool>,
    started_at_gte: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[tracing::instrument(name = "Listing a user's action tracks", skip(db, user))]
#[get("")]
pub async fn list_action_tracks(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let mut filters = ActionTrackQuery::get_default_filters();
            filters.show_inactive = !query.active_only.unwrap_or(false);
            filters.started_at_gte = query.started_at_gte;
            match ActionTrackQuery::find_by_user_id_with_filters(&db, user.id, filters).await {
                Ok(action_tracks) => HttpResponse::Ok().json(action_tracks),
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}

#[cfg(test)]
mod tests {
    use ::types::ActionTrackVisible;
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use chrono::{DateTime, Duration, FixedOffset, TimeDelta};
    use sea_orm::{entity::prelude::*, DbErr};

    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_action_tracks))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track_0 = factory::action_track(user.id)
            .duration(Some(120))
            .action_id(action.id)
            .insert(&db)
            .await?;
        let action_track_1 = factory::action_track(user.id)
            .started_at(action_track_0.started_at + TimeDelta::seconds(1))
            .duration(Some(180))
            .action_id(action.id)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_action_tracks: Vec<ActionTrackVisible> = test::read_body_json(resp).await;

        let expected = vec![
            ActionTrackVisible::from(action_track_1),
            ActionTrackVisible::from(action_track_0),
        ];

        assert_eq!(returned_action_tracks.len(), expected.len());
        assert_eq!(returned_action_tracks[0], expected[0]);
        assert_eq!(returned_action_tracks[1], expected[1]);

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_active_only() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let _inactive_action_track = factory::action_track(user.id)
            .duration(Some(120))
            .action_id(action.id)
            .insert(&db)
            .await?;
        let active_action_track = factory::action_track(user.id)
            .started_at(_inactive_action_track.started_at + TimeDelta::seconds(1))
            .action_id(action.id)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get()
            .uri("/?active_only=true")
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_action_tracks: Vec<ActionTrackVisible> = test::read_body_json(resp).await;

        let expected = vec![ActionTrackVisible::from(active_action_track)];

        assert_eq!(returned_action_tracks.len(), expected.len());
        assert_eq!(returned_action_tracks[0], expected[0]);

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_started_at_gte() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let started_at_gte: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-03-27T00:00:00Z").unwrap();
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .started_at(started_at_gte)
            .duration(Some(120))
            .insert(&db)
            .await?;
        let _old_action_track = factory::action_track(user.id)
            .action_id(action.id)
            .started_at(started_at_gte - Duration::seconds(1))
            .duration(Some(120))
            .insert(&db)
            .await?;

        let req = test::TestRequest::get()
            .uri(&format!(
                "/?started_at_gte={}",
                started_at_gte.format("%Y-%m-%dT%H:%M:%SZ")
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_action_tracks: Vec<ActionTrackVisible> = test::read_body_json(resp).await;

        let expected = vec![ActionTrackVisible::from(action_track)];

        assert_eq!(returned_action_tracks.len(), expected.len());
        assert_eq!(returned_action_tracks[0], expected[0]);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
