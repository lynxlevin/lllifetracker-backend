use crate::{my_way::actions::types::ActionVisibleWithGoal, UseCaseError};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionJoin, ActionOrder, ActionQuery},
    Order::Asc,
};
use entities::user as user_entity;

pub async fn list_actions<'a>(
    user: user_entity::Model,
    action_adapter: ActionAdapter<'a>,
) -> Result<Vec<ActionVisibleWithGoal>, UseCaseError> {
    action_adapter
        .join_active_goal()
        .filter_eq_user(&user)
        .order_by_ordering_nulls_last(Asc)
        .order_by_created_at(Asc)
        .get_all_with_goal()
        .await
        .map(|actions| {
            actions
                .iter()
                .map(|action| ActionVisibleWithGoal::from(action))
                .collect::<Vec<_>>()
        })
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
