use crate::{
    entities::user as user_entity,
    services::objective::Mutation as ObjectiveMutation,
    startup::AppState,
    types::{self, ObjectiveVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    objective_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
}

#[tracing::instrument(name = "Updating an objective", skip(data, user, req, path_param))]
#[put("/{objective_id}")]
pub async fn update_objective(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ObjectiveMutation::update(
                &data.conn,
                path_param.objective_id,
                user.id,
                req.name.clone(),
            )
            .await
            {
                Some(result) => match result {
                    Ok(objective) => HttpResponse::Ok().json(ObjectiveVisible {
                        id: objective.id,
                        name: objective.name,
                        created_at: objective.created_at,
                        updated_at: objective.updated_at,
                    }),
                    Err(e) => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                        HttpResponse::InternalServerError().json(types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                },
                None => HttpResponse::NotFound().json(types::ErrorResponse {
                    error: "Objective with this id was not found".to_string(),
                }),
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}
