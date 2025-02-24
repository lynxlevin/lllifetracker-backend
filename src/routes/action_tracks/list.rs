use entities::user as user_entity;
use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use crate::{
    services::action_track_query::ActionTrackQuery,
};
use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct QueryParam {
    active_only: Option<bool>,
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
            match ActionTrackQuery::find_all_by_user_id(
                &db,
                user.id,
                query.active_only.unwrap_or(false),
            )
            .await
            {
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
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, DbErr};
    use ::types::ActionTrackWithActionName;

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
            .insert(&db)
            .await?;
        let action_track_1 = factory::action_track(user.id)
            .duration(Some(180))
            .action_id(Some(action.id))
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_action_tracks: Vec<ActionTrackWithActionName> =
            test::read_body_json(resp).await;

        let expected = vec![
            ActionTrackWithActionName {
                id: action_track_1.id,
                action_id: Some(action.id),
                action_name: Some(action.name),
                started_at: action_track_1.started_at,
                ended_at: action_track_1.ended_at,
                duration: action_track_1.duration,
            },
            ActionTrackWithActionName {
                id: action_track_0.id,
                action_id: None,
                action_name: None,
                started_at: action_track_0.started_at,
                ended_at: action_track_0.ended_at,
                duration: action_track_0.duration,
            },
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
            .insert(&db)
            .await?;
        let active_action_track = factory::action_track(user.id)
            .action_id(Some(action.id))
            .insert(&db)
            .await?;

        let req = test::TestRequest::get()
            .uri("/?active_only=true")
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_action_tracks: Vec<ActionTrackWithActionName> =
            test::read_body_json(resp).await;

        let expected = vec![ActionTrackWithActionName {
            id: active_action_track.id,
            action_id: Some(action.id),
            action_name: Some(action.name),
            started_at: active_action_track.started_at,
            ended_at: active_action_track.ended_at,
            duration: active_action_track.duration,
        }];

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
