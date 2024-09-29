use crate::{
    entities::user as user_entity,
    services::ambition::{Mutation as AmbitionMutation, NewAmbition},
    startup::AppState,
    types::{self, AmbitionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
    description: Option<String>,
}

#[tracing::instrument(name = "Creating an ambition", skip(data, user))]
#[post("")]
pub async fn create_ambition(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionMutation::create_with_tag(
                &data.conn,
                NewAmbition {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(ambition) => HttpResponse::Ok().json(AmbitionVisible {
                    id: ambition.id,
                    name: ambition.name,
                    description: ambition.description,
                    created_at: ambition.created_at,
                    updated_at: ambition.updated_at,
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
