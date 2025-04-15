use types::ActionTrackUpdateRequest;
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

#[tracing::instrument(name = "Updating an action track", skip(db, user))]
#[put("/{action_track_id}")]
pub async fn update_action_track(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackUpdateRequest>,
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
