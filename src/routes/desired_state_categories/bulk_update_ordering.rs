use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::{
    desired_state_category_mutation::DesiredStateCategoryMutation,
    desired_state_category_query::DesiredStateCategoryQuery,
};
use types::DesiredStateCategoryBulkUpdateOrderingRequest;

use crate::utils::{response_401, response_500};

/// Fuzzy Ordering Design Decision
/// Ordering doesn’t need to be correctly serialized in the backend
/// - Skipping some numbers is OK.
///     => So no need for any handling when deleting or archiving an desired_state_category.
/// - Ordering numbers can be larger than the number of desired_state_categories.
/// - Ordering number can be null, and null desired_state_categories will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those desired_state_categories will be sorted, it only happens when un-archiving an desired_state_category.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an desired_state_category.
/// Ordering numbers need only be updated on this endpoint.

#[tracing::instrument(
    name = "Bulk updating desired_state_category ordering",
    skip(db, user, req)
)]
#[put("/bulk_update_ordering")]
pub async fn bulk_update_desired_state_category_ordering(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateCategoryBulkUpdateOrderingRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let categories = match DesiredStateCategoryQuery::find_in_ids_by_user_id(
                &db,
                req.ordering.clone(),
                user.id,
            )
            .await
            {
                Ok(categories) => categories,
                Err(e) => return response_500(e),
            };

            let params = categories
                .into_iter()
                .map(|category| {
                    let ordering = req
                        .ordering
                        .iter()
                        .position(|id| id == &category.id)
                        .and_then(|ordering| Some((ordering + 1) as i32));
                    (category, ordering)
                })
                .collect::<Vec<_>>();

            match DesiredStateCategoryMutation::bulk_update_ordering(&db, params).await {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
