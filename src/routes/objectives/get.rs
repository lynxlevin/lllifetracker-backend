use crate::{
    entities::user as user_entity,
    services::objective::Query as ObjectiveQuery,
    startup::AppState,
    types::{self, ObjectiveVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    objective_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an objective", skip(data, user))]
#[get("/{objective_id}")]
pub async fn get_objective(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ObjectiveQuery::find_by_id_and_user_id(
                &data.conn,
                path_param.objective_id,
                user.id,
            )
            .await
            {
                Ok(objective) => match objective {
                    Some(objective) => HttpResponse::Ok().json(ObjectiveVisible {
                        id: objective.id,
                        name: objective.name,
                        created_at: objective.created_at,
                        updated_at: objective.updated_at,
                    }),
                    None => HttpResponse::NotFound().json(types::ErrorResponse {
                        error: "Objective with this id was not found".to_string(),
                    }),
                },
                Err(e) => {
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
