use ::types::{self, ActionTrackVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::{sea_orm_active_enums::ActionTrackType, user as user_entity};
use sea_orm::{DbConn, DbErr};
use services::{
    action_query::ActionQuery,
    action_track_mutation::{ActionTrackMutation, NewActionTrack},
};
use types::CustomDbErr;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    started_at: chrono::DateTime<chrono::FixedOffset>,
    action_id: uuid::Uuid,
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
            match ActionQuery::find_by_id_and_user_id(&db, req.action_id, user.id).await {
                Ok(action) => {
                    let result = match action.track_type {
                        ActionTrackType::TimeSpan => {
                            ActionTrackMutation::create(
                                &db,
                                NewActionTrack {
                                    started_at: req.started_at,
                                    ended_at: None,
                                    action_id: req.action_id,
                                    user_id: user.id,
                                },
                            )
                            .await
                        }
                        ActionTrackType::Count => {
                            ActionTrackMutation::create(
                                &db,
                                NewActionTrack {
                                    started_at: req.started_at,
                                    ended_at: Some(req.started_at),
                                    action_id: req.action_id,
                                    user_id: user.id,
                                },
                            )
                            .await
                        }
                    };
                    match result {
                        Ok(action_track) => {
                            let res: ActionTrackVisible = action_track.into();
                            HttpResponse::Created().json(res)
                        }
                        Err(e) => {
                            match &e {
                                DbErr::Custom(message) => {
                                    match message.parse::<CustomDbErr>().unwrap() {
                                        CustomDbErr::Duplicate => {
                                            return HttpResponse::Conflict().json(
                                                types::ErrorResponse {
                                                    error: "A track for the same action which starts at the same time exists.".to_string(),
                                                }
                                            )
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                            HttpResponse::InternalServerError().json(types::ErrorResponse {
                                error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                            })
                        }
                    }
                }
                Err(e) => {
                    match &e {
                        DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                return HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "An action with that id does not exist.".to_string(),
                                })
                            }
                            _ => {}
                        },
                        _ => {}
                    }
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
    use chrono::{SubsecRound, Utc};
    use sea_orm::{entity::prelude::ActiveModelTrait, DbErr, EntityTrait};

    use entities::{action_track, sea_orm_active_enums::ActionTrackType};
    use test_utils::{self, *};

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
    async fn happy_path_time_span_type() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let app = init_app(db.clone()).await;

        let started_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                started_at,
                action_id: action.id,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_action_track: ActionTrackVisible = test::read_body_json(res).await;
        assert_eq!(returned_action_track.action_id, action.id);
        assert_eq!(
            returned_action_track.started_at,
            started_at.trunc_subsecs(0)
        );
        assert_eq!(returned_action_track.ended_at, None);
        assert_eq!(returned_action_track.duration, None);

        let created_action_track = action_track::Entity::find_by_id(returned_action_track.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_action_track.user_id, user.id);
        assert_eq!(created_action_track.action_id, action.id);
        assert_eq!(created_action_track.started_at, started_at.trunc_subsecs(0));
        assert_eq!(created_action_track.ended_at, None);
        assert_eq!(created_action_track.duration, None);

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_count_type() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id)
            .track_type(ActionTrackType::Count)
            .insert(&db)
            .await?;
        let app = init_app(db.clone()).await;

        let started_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                started_at,
                action_id: action.id,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_action_track: ActionTrackVisible = test::read_body_json(res).await;
        assert_eq!(returned_action_track.action_id, action.id);
        assert_eq!(
            returned_action_track.started_at,
            started_at.trunc_subsecs(0)
        );
        assert_eq!(
            returned_action_track.ended_at,
            Some(started_at.trunc_subsecs(0))
        );
        assert_eq!(returned_action_track.duration, Some(0));

        let created_action_track = action_track::Entity::find_by_id(returned_action_track.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_action_track.user_id, user.id);
        assert_eq!(
            ActionTrackVisible::from(created_action_track),
            returned_action_track
        );

        Ok(())
    }

    #[actix_web::test]
    async fn conflict_on_duplicate_creation() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let existing_action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                started_at: existing_action_track.started_at,
                action_id: existing_action_track.action_id,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CONFLICT);

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
                action_id: uuid::Uuid::now_v7(),
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
