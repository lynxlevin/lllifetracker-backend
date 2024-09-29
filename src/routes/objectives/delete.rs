use crate::{
    entities::user as user_entity,
    services::objective::Mutation as ObjectiveMutation,
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
    objective_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an objective", skip(data, user, path_param))]
#[delete("/{objective_id}")]
pub async fn delete_objective(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ObjectiveMutation::delete(&data.conn, path_param.objective_id, user.id).await {
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
