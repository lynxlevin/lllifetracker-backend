use db_adapters::{
    desired_state_category_adapter::{
        DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryOrder,
        DesiredStateCategoryQuery,
    },
    Order::Asc,
};
use entities::user as user_entity;

use crate::{my_way::desired_state_categories::types::DesiredStateCategoryVisible, UseCaseError};

pub async fn list_desired_state_categories<'a>(
    user: user_entity::Model,
    category_adapter: DesiredStateCategoryAdapter<'a>,
) -> Result<Vec<DesiredStateCategoryVisible>, UseCaseError> {
    match category_adapter
        .filter_eq_user(&user)
        .order_by_ordering_nulls_last(Asc)
        .order_by_id(Asc)
        .get_all()
        .await
    {
        Ok(categories) => Ok(categories
            .iter()
            .map(|category| DesiredStateCategoryVisible::from(category))
            .collect::<Vec<_>>()),
        Err(e) => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
    }
}
