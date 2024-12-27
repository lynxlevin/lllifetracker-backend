use crate::{
    entities::user as user_entity,
    services::action_track_mutation::{ActionTrackMutation, UpdateActionTrack},
    types::{self, ActionTrackVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_track_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    action_id: Option<uuid::Uuid>,
    started_at: chrono::DateTime<chrono::FixedOffset>,
    ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[tracing::instrument(name = "Updating an action track", skip(db, user))]
#[put("/{action_track_id}")]
pub async fn update_action_track(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionTrackMutation::update(
                &db,
                path_param.action_track_id,
                UpdateActionTrack {
                    started_at: req.started_at,
                    ended_at: req.ended_at,
                    action_id: req.action_id,
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(action_track) => HttpResponse::Ok().json(ActionTrackVisible {
                    id: action_track.id,
                    action_id: action_track.action_id,
                    started_at: action_track.started_at,
                    ended_at: action_track.ended_at,
                    duration: action_track.duration,
                }),
                Err(e) => match e {
                    DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "ActionTrack with this id was not found".to_string(),
                            })
                        }
                    },
                    e => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                        HttpResponse::InternalServerError().json(types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                },
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test, App, HttpMessage,
    };
    use chrono::Utc;
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait};

    use crate::{entities::action_track, test_utils::{self, factory}};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(update_action_track)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = test_utils::seed::create_action_track(&db, None, None, user.id).await?;
        let ended_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let duration = 180;
        let started_at = ended_at - chrono::TimeDelta::seconds(duration.into());

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action_track.id))
            .set_json(RequestBody {
                action_id: Some(action.id),
                started_at,
                ended_at: Some(ended_at),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_action_track: ActionTrackVisible = test::read_body_json(res).await;
        assert_eq!(returned_action_track.id, action_track.id);
        assert_eq!(returned_action_track.action_id, Some(action.id));
        assert_eq!(returned_action_track.started_at, started_at);
        assert_eq!(returned_action_track.ended_at, Some(ended_at));
        assert_eq!(returned_action_track.duration, Some(duration));

        let updated_action_track = action_track::Entity::find_by_id(action_track.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_action_track.action_id, Some(action.id));
        assert_eq!(updated_action_track.user_id, user.id);
        assert_eq!(updated_action_track.started_at, started_at);
        assert_eq!(updated_action_track.ended_at, Some(ended_at));
        assert_eq!(updated_action_track.duration, Some(duration));

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action_track = test_utils::seed::create_action_track(&db, None, None, user.id).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action_track.id))
            .set_json(RequestBody {
                action_id: None,
                started_at: Utc::now().into(),
                ended_at: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
