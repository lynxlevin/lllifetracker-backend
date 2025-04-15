use entities::user as user_entity;
use types::ActionBulkUpdateOrderRequest;
use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use services::action_mutation::ActionMutation;
use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

/// Fuzzy Ordering Design Decision
/// Ordering doesn’t need to be correctly serialized in the backend
/// - Skipping some numbers is OK.
///     => So no need for any handling when deleting or archiving an action.
/// - Ordering numbers can be larger than the number of actions.
/// - Ordering number can be null, and null actions will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those actions will be sorted, it only happens when un-archiving an action.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an action.
/// Ordering numbers need only be updated on this endpoint.

#[tracing::instrument(name = "Bulk updating action ordering", skip(db, user, req))]
#[put("/bulk_update_ordering")]
pub async fn bulk_update_action_ordering(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionBulkUpdateOrderRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::bulk_update_ordering(&db, user.id, req.ordering.clone()).await {
                Ok(_) => HttpResponse::Ok().finish(),
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
