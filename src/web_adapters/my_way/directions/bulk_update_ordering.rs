use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::direction_adapter::DirectionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::directions::{
    bulk_update_ordering::bulk_update_direction_ordering,
    types::DirectionBulkUpdateOrderingRequest,
};

use crate::utils::{response_401, response_500};

/// Fuzzy Ordering Design Decision
/// Ordering doesn’t need to be correctly serialized in the backend
/// - Skipping some numbers is OK.
///     => So no need for any handling when deleting or archiving an direction.
/// - Ordering numbers can be larger than the number of directions.
/// - Ordering number can be null, and null directions will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those directions will be sorted, it only happens when un-archiving an direction.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an direction.
/// Ordering numbers need only be updated on this endpoint.

#[tracing::instrument(name = "Bulk updating direction ordering", skip(db, user, req))]
#[put("/bulk_update_ordering")]
pub async fn bulk_update_direction_ordering_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DirectionBulkUpdateOrderingRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match bulk_update_direction_ordering(
                user.into_inner(),
                req.into_inner(),
                DirectionAdapter::init(&db),
            )
            .await
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
