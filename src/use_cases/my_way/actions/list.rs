use crate::{
    my_way::actions::types::{ActionListQuery, ActionVisible},
    UseCaseError,
};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionOrder, ActionQuery},
    Order::Asc,
};
use entities::user as user_entity;

pub async fn list_actions<'a>(
    user: user_entity::Model,
    params: ActionListQuery,
    action_adapter: ActionAdapter<'a>,
) -> Result<Vec<ActionVisible>, UseCaseError> {
    action_adapter
        .filter_eq_user(&user)
        .filter_eq_archived(params.show_archived_only.unwrap_or(false))
        .order_by_ordering_nulls_last(Asc)
        .order_by_created_at(Asc)
        .get_all()
        .await
        .map(|actions| {
            actions
                .iter()
                .map(|action| ActionVisible::from(action))
                .collect::<Vec<_>>()
        })
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
