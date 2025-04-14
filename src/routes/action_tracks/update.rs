use ::types::{self, ActionTrackVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::action_track_mutation::{ActionTrackMutation, UpdateActionTrack};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_track_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    action_id: uuid::Uuid,
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
                Ok(action_track) => {
                    let res: ActionTrackVisible = action_track.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => {
                    match &e {
                        DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                return HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "ActionTrack with this id was not found".to_string(),
                                })
                            }
                            CustomDbErr::Duplicate => {
                                return HttpResponse::Conflict().json(types::ErrorResponse {
                                    error:
                                        "A track for the same action which starts at the same time exists."
                                            .to_string(),
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
    use chrono::{SubsecRound, TimeDelta, Utc};
    use sea_orm::{entity::prelude::ActiveModelTrait, DbErr, EntityTrait};

    use entities::action_track;
    use test_utils::{self, *};

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
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;
        let ended_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let duration = 180;
        let started_at = ended_at - chrono::TimeDelta::seconds(duration.into());

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action_track.id))
            .set_json(RequestBody {
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
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
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
            .uri(&format!("/{}", action_track.id))
            .set_json(RequestBody {
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
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action_track.id))
            .set_json(RequestBody {
                action_id: uuid::Uuid::now_v7(),
                started_at: Utc::now().into(),
                ended_at: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
