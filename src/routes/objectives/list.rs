use crate::{
    entities::user as user_entity,
    services::objective::Query as ObjectiveQuery,
    startup::AppState,
    types::{self, ObjectiveVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};

#[tracing::instrument(name = "Listing a user's objectives", skip(data, user))]
#[get("")]
pub async fn list_objectives(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ObjectiveQuery::find_all_by_user_id(&data.conn, user.id).await {
                Ok(objectives) => HttpResponse::Ok().json(
                    objectives
                        .iter()
                        .map(|objective| ObjectiveVisible {
                            id: objective.id,
                            name: objective.name.clone(),
                            created_at: objective.created_at,
                            updated_at: objective.updated_at,
                        })
                        .collect::<Vec<ObjectiveVisible>>(),
                ),
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
