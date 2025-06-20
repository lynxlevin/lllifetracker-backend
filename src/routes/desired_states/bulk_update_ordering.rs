use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_adapter::{
    DesiredStateAdapter, DesiredStateFilter, DesiredStateMutation, DesiredStateQuery,
};
use entities::user as user_entity;
use sea_orm::DbConn;

use types::DesiredStateBulkUpdateOrderingRequest;

use crate::utils::{response_401, response_500};

/// Fuzzy Ordering Design Decision
/// Ordering doesn’t need to be correctly serialized in the backend
/// - Skipping some numbers is OK.
///     => So no need for any handling when deleting or archiving an desired_state.
/// - Ordering numbers can be larger than the number of desired_states.
/// - Ordering number can be null, and null desired_states will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those desired_states will be sorted, it only happens when un-archiving an desired_state.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an desired_state.
/// Ordering numbers need only be updated on this endpoint.

#[tracing::instrument(name = "Bulk updating desired_state ordering", skip(db, user, req))]
#[put("/bulk_update_ordering")]
pub async fn bulk_update_desired_state_ordering(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateBulkUpdateOrderingRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let desired_states = match DesiredStateAdapter::init(&db)
                .filter_eq_user(&user)
                .filter_in_ids(req.ordering.clone())
                .get_all()
                .await
            {
                Ok(desired_states) => desired_states,
                Err(e) => return response_500(e),
            };
            match DesiredStateAdapter::init(&db)
                .bulk_update_ordering(desired_states, req.ordering.clone())
                .await
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
