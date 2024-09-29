use crate::{
    entities::user as user_entity,
    services::ambition::Query as AmbitionQuery,
    startup::AppState,
    types::{self, AmbitionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};

#[tracing::instrument(name = "Listing a user's ambitions", skip(data, user))]
#[get("")]
pub async fn list_ambitions(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionQuery::find_all_by_user_id(&data.conn, user.id).await {
                Ok(ambitions) => HttpResponse::Ok().json(
                    ambitions
                        .iter()
                        .map(|ambition| AmbitionVisible {
                            id: ambition.id,
                            name: ambition.name.clone(),
                            description: ambition.description.clone(),
                            created_at: ambition.created_at,
                            updated_at: ambition.updated_at,
                        })
                        .collect::<Vec<AmbitionVisible>>(),
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