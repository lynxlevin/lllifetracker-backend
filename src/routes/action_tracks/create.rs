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
use types::{ActionTrackCreateRequest, CustomDbErr};

#[tracing::instrument(name = "Creating an action track", skip(db, user))]
#[post("")]
pub async fn create_action_track(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackCreateRequest>,
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
