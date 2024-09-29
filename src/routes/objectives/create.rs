use crate::{
    entities::user as user_entity,
    services::objective::{Mutation as ObjectiveMutation, NewObjective},
    startup::AppState,
    types::{self, ObjectiveVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
}

#[tracing::instrument(name = "Creating an objective", skip(data, user))]
#[post("")]
pub async fn create_objective(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ObjectiveMutation::create_with_tag(
                &data.conn,
                NewObjective {
                    name: req.name.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
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
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}
