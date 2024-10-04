use crate::{
    entities::user as user_entity,
    services::action::Query as ActionQuery,
    types::{self, ActionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing a user's actions", skip(db, user))]
#[get("")]
pub async fn list_actions(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionQuery::find_all_by_user_id(&db, user.id).await {
                Ok(actions) => HttpResponse::Ok().json(
                    actions
                        .iter()
                        .map(|action| ActionVisible {
                            id: action.id,
                            name: action.name.clone(),
                            created_at: action.created_at,
                            updated_at: action.updated_at,
                        })
                        .collect::<Vec<ActionVisible>>(),
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
