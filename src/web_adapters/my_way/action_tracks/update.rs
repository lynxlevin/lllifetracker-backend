use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::action_track_service::ActionTrackService;

use common::errors::error_responses::{response_400, response_401, response_404, response_500};
use entities::user as user_entity;
use use_cases::my_way::action_tracks::{
    types::ActionTrackUpdateRequest,
    update::{update_action_track, UpdateActionTracksError},
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_track_id: uuid::Uuid,
}

#[put("/{action_track_id}")]
pub async fn update_action_track_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_action_track(
                user.into_inner(),
                req.into_inner(),
                path_param.action_track_id,
                ActionTrackService::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UpdateActionTracksError::ValidationError(message) => response_400(message),
                    UpdateActionTracksError::NotFound(message) => response_404(message),
                    UpdateActionTracksError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
