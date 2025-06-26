use crate::{
    my_way::desired_state_categories::types::DesiredStateCategoryBulkUpdateOrderingRequest,
    UseCaseError,
};
use db_adapters::desired_state_category_adapter::{
    DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryMutation,
    DesiredStateCategoryQuery,
};
use entities::user as user_entity;

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

pub async fn bulk_update_desired_state_category_ordering<'a>(
    user: user_entity::Model,
    params: DesiredStateCategoryBulkUpdateOrderingRequest,
    category_adapter: DesiredStateCategoryAdapter<'a>,
) -> Result<(), UseCaseError> {
    let categories = category_adapter
        .clone()
        .filter_eq_user(&user)
        .filter_in_ids(params.ordering.clone())
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let params = categories
        .into_iter()
        .map(|category| {
            let ordering = params
                .ordering
                .iter()
                .position(|id| id == &category.id)
                .and_then(|ordering| Some((ordering + 1) as i32));
            (category, ordering)
        })
        .collect::<Vec<_>>();

    category_adapter
        .bulk_update_ordering(params)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
