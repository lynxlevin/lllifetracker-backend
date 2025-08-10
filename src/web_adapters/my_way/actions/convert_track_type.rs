use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::{action_adapter::ActionAdapter, action_goal_adapter::ActionGoalAdapter};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    my_way::actions::{
        convert_track_type::convert_action_track_type, types::ActionTrackTypeConversionRequest,
    },
    UseCaseError,
};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Converting action type", skip(db, user, req, path_param))]
#[put("/{action_id}/track_type")]
pub async fn convert_action_track_type_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackTypeConversionRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match convert_action_track_type(
                user.into_inner(),
                req.into_inner(),
                path_param.action_id,
                ActionAdapter::init(&db),
                ActionGoalAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
