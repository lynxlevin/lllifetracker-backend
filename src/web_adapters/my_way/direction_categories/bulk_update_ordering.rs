use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::direction_category_adapter::DirectionCategoryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::direction_categories::{
    bulk_update_ordering::bulk_update_direction_category_ordering,
    types::DirectionCategoryBulkUpdateOrderingRequest,
};

use crate::utils::{response_401, response_500};

/// Fuzzy Ordering Design Decision
/// Ordering doesn’t need to be correctly serialized in the backend
/// - Skipping some numbers is OK.
///     => So no need for any handling when deleting or archiving an direction_category.
/// - Ordering numbers can be larger than the number of direction_categories.
/// - Ordering number can be null, and null direction_categories will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those direction_categories will be sorted, it only happens when un-archiving an direction_category.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an direction_category.
/// Ordering numbers need only be updated on this endpoint.

#[tracing::instrument(
    name = "Bulk updating direction_category ordering",
    skip(db, user, req)
)]
#[put("/bulk_update_ordering")]
pub async fn bulk_update_direction_category_ordering_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DirectionCategoryBulkUpdateOrderingRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match bulk_update_direction_category_ordering(
                user.into_inner(),
                req.into_inner(),
                DirectionCategoryAdapter::init(&db),
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
