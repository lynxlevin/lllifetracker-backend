use crate::{
    entities::user as user_entity,
    services::action::Mutation as ActionMutation,
    startup::AppState,
    types::{self, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an action", skip(data, user, path_param))]
#[delete("/{action_id}")]
pub async fn delete_action(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::delete(&data.conn, path_param.action_id, user.id).await {
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