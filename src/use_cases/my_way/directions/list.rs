use db_adapters::{
    direction_adapter::{
        DirectionAdapter, DirectionFilter, DirectionJoin, DirectionOrder,
        DirectionQuery,
    },
    Order::Asc,
};
use entities::user as user_entity;

use crate::{
    my_way::directions::types::{DirectionListQuery, DirectionVisible},
    UseCaseError,
};

pub async fn list_directions<'a>(
    user: user_entity::Model,
    params: DirectionListQuery,
    direction_adapter: DirectionAdapter<'a>,
) -> Result<Vec<DirectionVisible>, UseCaseError> {
    match direction_adapter
        .join_category()
        .filter_eq_user(&user)
        .filter_eq_archived(params.show_archived_only.unwrap_or(false))
        .order_by_category_ordering_nulls_last(Asc)
        .order_by_ordering_nulls_first(Asc)
        .order_by_created_at(Asc)
        .get_all()
        .await
    {
        Ok(directions) => Ok(directions
            .iter()
            .map(|direction| DirectionVisible::from(direction))
            .collect::<Vec<_>>()),
        Err(e) => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
    }
}
