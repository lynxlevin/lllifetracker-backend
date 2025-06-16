use ::types::{self, ActionTrackVisible};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use chrono::SubsecRound;
use db_adapters::{
    action_track_mutation::{ActionTrackMutation, UpdateActionTrackParams},
    action_track_query::{ActionTrackQuery, ActionTrackQueryFilter},
    CustomDbErr,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
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
            let action_track = match ActionTrackQuery::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.action_track_id)
                .await
            {
                Ok(action_track) => match action_track {
                    Some(action_track) => action_track,
                    None => return response_404("ActionTrack with this id was not found"),
                },
                Err(e) => return response_500(e),
            };
            match ActionTrackMutation::init(&db)
                .update(
                    action_track,
                    UpdateActionTrackParams {
                        started_at: req.started_at.trunc_subsecs(0),
                        ended_at: req
                            .ended_at
                            .and_then(|ended_at| Some(ended_at.trunc_subsecs(0))),
                        duration: req.ended_at.and_then(|ended_at| {
                            Some(
                                (ended_at.trunc_subsecs(0) - req.started_at.trunc_subsecs(0))
                                    .num_seconds(),
                            )
                        }),
                        action_id: req.action_id,
                    },
                )
                .await
            {
                Ok(action_track) => HttpResponse::Ok().json(ActionTrackVisible::from(action_track)),
                Err(e) => match &e {
                    DbErr::Custom(ce) => match CustomDbErr::from(ce) {
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
