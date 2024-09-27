use crate::{
    entities::user as user_entity,
    services::action::{Mutation as ActionMutation, NewAction},
    startup::AppState,
    types::{self, ActionVisible},
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

#[tracing::instrument(name = "Creating an action", skip(data, user))]
#[post("")]
pub async fn create_action(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::create(
                &data.conn,
                NewAction {
                    name: req.name.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(action) => HttpResponse::Ok().json(ActionVisible {
                    id: action.id,
                    name: action.name,
                    created_at: action.created_at,
                    updated_at: action.updated_at,
                }),
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: "Something unexpected happened. Kindly try again".to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}
