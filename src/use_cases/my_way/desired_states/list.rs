use db_adapters::{
    desired_state_adapter::{
        DesiredStateAdapter, DesiredStateFilter, DesiredStateJoin, DesiredStateOrder,
        DesiredStateQuery,
    },
    Order::Asc,
};
use entities::user as user_entity;

use crate::{
    my_way::desired_states::types::{DesiredStateListQuery, DesiredStateVisible},
    UseCaseError,
};

pub async fn list_desired_states<'a>(
    user: user_entity::Model,
    params: DesiredStateListQuery,
    desired_state_adapter: DesiredStateAdapter<'a>,
) -> Result<Vec<DesiredStateVisible>, UseCaseError> {
    match desired_state_adapter
        .join_category()
        .filter_eq_user(&user)
        .filter_eq_archived(params.show_archived_only.unwrap_or(false))
        .order_by_category_ordering_nulls_last(Asc)
        .order_by_ordering_nulls_first(Asc)
        .order_by_created_at(Asc)
        .get_all()
        .await
    {
        Ok(desired_states) => Ok(desired_states
            .iter()
            .map(|desired_state| DesiredStateVisible::from(desired_state))
            .collect::<Vec<_>>()),
        Err(e) => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
    }
}
