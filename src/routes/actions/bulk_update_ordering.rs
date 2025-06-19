use ::types;
use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery};
use entities::user as user_entity;
use sea_orm::DbConn;
use types::ActionBulkUpdateOrderRequest;

use crate::utils::{response_401, response_500};

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
            let actions = match ActionAdapter::init(&db)
                .filter_eq_user(&user)
                .filter_in_ids(req.ordering.clone())
                .get_all()
                .await
            {
                Ok(actions) => actions,
                Err(e) => return response_500(e),
            };
            match ActionAdapter::init(&db)
                .bulk_update_ordering(actions, req.ordering.clone())
                .await
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
