use entities::user as user_entity;
use types::AmbitionBulkUpdateOrderingRequest;
use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use services::ambition_mutation::AmbitionMutation;
use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;


/// Fuzzy Ordering Design Decision
/// Ordering doesn’t need to be correctly serialized in the backend
/// - Skipping some numbers is OK.
///     => So no need for any handling when deleting or archiving an ambition.
/// - Ordering numbers can be larger than the number of ambitions.
/// - Ordering number can be null, and null ambitions will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those ambitions will be sorted, it only happens when un-archiving an ambition.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an ambition.
/// Ordering numbers need only be updated on this endpoint.

#[tracing::instrument(name = "Bulk updating ambition ordering", skip(db, user, req))]
#[put("/bulk_update_ordering")]
pub async fn bulk_update_ambition_ordering(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<AmbitionBulkUpdateOrderingRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionMutation::bulk_update_ordering(&db, user.id, req.ordering.clone()).await {
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
