use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::action_service::ActionService;
use entities::user as user_entity;
use use_cases::my_way::actions::{
    bulk_update_ordering::{bulk_update_action_ordering, BulkUpdateOrderingActionError},
    types::ActionBulkUpdateOrderRequest,
};

use common::errors::error_responses::{response_401, response_500};

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

#[put("/bulk_update_ordering")]
pub async fn bulk_update_action_ordering_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionBulkUpdateOrderRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match bulk_update_action_ordering(user.into_inner(), req.into_inner(), ActionService::init(&db)).await
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => match e {
                    BulkUpdateOrderingActionError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
