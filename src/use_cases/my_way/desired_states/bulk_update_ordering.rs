use db_adapters::desired_state_adapter::{
    DesiredStateAdapter, DesiredStateFilter, DesiredStateMutation, DesiredStateQuery,
};
use entities::user as user_entity;

use crate::{my_way::desired_states::types::DesiredStateBulkUpdateOrderingRequest, UseCaseError};

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

pub async fn bulk_update_desired_state_ordering<'a>(
    user: user_entity::Model,
    params: DesiredStateBulkUpdateOrderingRequest,
    desired_state_adapter: DesiredStateAdapter<'a>,
) -> Result<(), UseCaseError> {
    let desired_states = desired_state_adapter
        .clone()
        .filter_eq_user(&user)
        .filter_in_ids(params.ordering.clone())
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    desired_state_adapter
        .bulk_update_ordering(desired_states, params.ordering.clone())
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
