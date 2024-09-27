use crate::{
    entities::user as user_entity,
    services::action::Query as ActionQuery,
    startup::AppState,
    types::{self, ActionVisible},
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};

#[tracing::instrument(name = "Listing a user's actions", skip(data, user))]
#[get("")]
pub async fn list_actions(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::ActiveModel>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionQuery::find_all_by_user_id(&data.conn, user.id.unwrap()).await {
                Ok(actions) => HttpResponse::Ok().json(
                    actions
                        .iter()
                        .map(|a| ActionVisible {
                            id: a.id,
                            name: a.name.clone(),
                            created_at: a.created_at,
                            updated_at: a.updated_at,
                        })
                        .collect::<Vec<ActionVisible>>(),
                ),
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
