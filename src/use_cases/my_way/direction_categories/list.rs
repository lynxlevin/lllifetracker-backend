use db_adapters::{
    direction_category_adapter::{
        DirectionCategoryAdapter, DirectionCategoryFilter, DirectionCategoryOrder,
        DirectionCategoryQuery,
    },
    Order::Asc,
};
use entities::user as user_entity;

use crate::{my_way::direction_categories::types::DirectionCategoryVisible, UseCaseError};

pub async fn list_direction_categories<'a>(
    user: user_entity::Model,
    category_adapter: DirectionCategoryAdapter<'a>,
) -> Result<Vec<DirectionCategoryVisible>, UseCaseError> {
    match category_adapter
        .filter_eq_user(&user)
        .order_by_ordering_nulls_last(Asc)
        .order_by_id(Asc)
        .get_all()
        .await
    {
        Ok(categories) => Ok(categories
            .iter()
            .map(|category| DirectionCategoryVisible::from(category))
            .collect::<Vec<_>>()),
        Err(e) => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
    }
}
