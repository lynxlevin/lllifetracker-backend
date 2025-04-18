use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::action_track_mutation::ActionTrackMutation;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_track_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an action track", skip(db, user))]
#[delete("/{action_track_id}")]
pub async fn delete_action_track(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionTrackMutation::delete(&db, path_param.action_track_id, user.id).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => {
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
