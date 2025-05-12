use ::types::{self, ActionTrackVisible, CustomDbErr};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::action_track_mutation::{ActionTrackMutation, UpdateActionTrack};
use types::ActionTrackUpdateRequest;

use crate::utils::{response_401, response_404, response_409, response_500};

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
                Err(e) => match &e {
                    DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            response_404("ActionTrack with this id was not found")
                        }
                        CustomDbErr::Duplicate => response_409(
                            "A track for the same action which starts at the same time exists.",
                        ),
                        _ => response_500(e),
                    },
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
