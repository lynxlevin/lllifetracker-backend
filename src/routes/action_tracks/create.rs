use entities::user as user_entity;
use crate::{
    services::action_track_mutation::{ActionTrackMutation, NewActionTrack},
    types::{self, ActionTrackVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    started_at: chrono::DateTime<chrono::FixedOffset>,
    action_id: Option<uuid::Uuid>,
}

#[tracing::instrument(name = "Creating an action track", skip(db, user))]
#[post("")]
pub async fn create_action_track(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionTrackMutation::create(
                &db,
                NewActionTrack {
                    started_at: req.started_at,
                    action_id: req.action_id,
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(action_track) => {
                    let res: ActionTrackVisible = action_track.into();
                    HttpResponse::Created().json(res)
                }
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
    use chrono::Utc;
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait};

    use entities::action_track;
    use crate::{
        test_utils::{self, *},
    };

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(create_action_track))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let app = init_app(db.clone()).await;

        let started_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                started_at,
                action_id: Some(action.id),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_action_track: ActionTrackVisible = test::read_body_json(res).await;
        assert_eq!(returned_action_track.action_id, Some(action.id));
        assert_eq!(returned_action_track.started_at, started_at);
        assert_eq!(returned_action_track.ended_at, None);
        assert_eq!(returned_action_track.duration, None);

        let created_action_track = action_track::Entity::find_by_id(returned_action_track.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_action_track.user_id, user.id);
        assert_eq!(created_action_track.action_id, Some(action.id));
        assert_eq!(created_action_track.started_at, started_at);
        assert_eq!(created_action_track.ended_at, None);
        assert_eq!(created_action_track.duration, None);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                started_at: Utc::now().into(),
                action_id: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
