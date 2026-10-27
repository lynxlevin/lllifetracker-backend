use crate::my_way::actions::types::ActionBulkUpdateOrderRequest;
use domain_services::action_service::{
    ActionBulkUpdateOrderingParams, ActionService, ActionServiceError, ActionServiceMutation,
};
use entities::user as user_entity;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BulkUpdateOrderingActionError {
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionServiceError> for BulkUpdateOrderingActionError {
    fn from(e: ActionServiceError) -> Self {
        match e {
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

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

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
    ),
    skip_all
)]
pub async fn bulk_update_action_ordering<'a>(
    user: user_entity::Model,
    params: ActionBulkUpdateOrderRequest,
    action_service: ActionService<'a>,
) -> Result<(), BulkUpdateOrderingActionError> {
    action_service
        .bulk_update_ordering(ActionBulkUpdateOrderingParams {
            user_id: user.id,
            action_ids_in_order: params.ordering,
        })
        .await?;

    Ok(())
}
