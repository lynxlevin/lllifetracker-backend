use db_adapters::direction_adapter::{
    DirectionAdapter, DirectionFilter, DirectionMutation, DirectionQuery,
};
use entities::user as user_entity;

use crate::{my_way::directions::types::DirectionBulkUpdateOrderingRequest, UseCaseError};

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

pub async fn bulk_update_direction_ordering<'a>(
    user: user_entity::Model,
    params: DirectionBulkUpdateOrderingRequest,
    direction_adapter: DirectionAdapter<'a>,
) -> Result<(), UseCaseError> {
    let directions = direction_adapter
        .clone()
        .filter_eq_user(&user)
        .filter_in_ids(params.ordering.clone())
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    direction_adapter
        .bulk_update_ordering(directions, params.ordering.clone())
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
