use crate::{my_way::ambitions::types::AmbitionBulkUpdateOrderingRequest, UseCaseError};
use db_adapters::ambition_adapter::{
    AmbitionAdapter, AmbitionFilter, AmbitionMutation, AmbitionQuery,
};
use entities::user as user_entity;

/// Fuzzy Ordering Design Decision
/// Ordering doesn’t need to be correctly serialized in the backend
/// - Skipping some numbers is OK.
///     => So no need for any handling when deleting or archiving an ambition.
/// - Ordering numbers can be larger than the number of ambitions.
/// - Ordering number can be null, and null ambitions will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those ambitions will be sorted, it only happens when un-archiving an ambition.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an ambition.
/// Ordering numbers need only be updated on this endpoint.

pub async fn bulk_update_ambition_ordering<'a>(
    user: user_entity::Model,
    params: AmbitionBulkUpdateOrderingRequest,
    ambition_adapter: AmbitionAdapter<'a>,
) -> Result<(), UseCaseError> {
    let ambitions = ambition_adapter
        .clone()
        .filter_eq_user(&user)
        .filter_in_ids(params.ordering.clone())
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    ambition_adapter
        .bulk_update_ordering(ambitions, params.ordering.clone())
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
