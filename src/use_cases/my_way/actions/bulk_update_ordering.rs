use crate::{my_way::actions::types::ActionBulkUpdateOrderRequest, UseCaseError};
use db_adapters::action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery};
use entities::user as user_entity;

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

pub async fn bulk_update_action_ordering<'a>(
    user: user_entity::Model,
    params: ActionBulkUpdateOrderRequest,
    action_adapter: ActionAdapter<'a>,
) -> Result<(), UseCaseError> {
    let actions = action_adapter
        .clone()
        .filter_eq_user(&user)
        .filter_in_ids(params.ordering.clone())
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    action_adapter
        .bulk_update_ordering(actions, params.ordering.clone())
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
