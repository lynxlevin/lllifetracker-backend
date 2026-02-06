use db_adapters::{
    tag_adapter::{TagAdapter, TagFilter, TagJoin, TagOrder, TagQuery},
    Order::Asc,
};
use entities::user as user_entity;

use crate::{tags::types::TagVisible, UseCaseError};

pub async fn list_tags<'a>(
    user: user_entity::Model,
    tag_adapter: TagAdapter<'a>,
) -> Result<Vec<TagVisible>, UseCaseError> {
    tag_adapter
        .join_ambition()
        .join_direction()
        .join_action()
        .filter_eq_user(&user)
        .order_by_type(Asc)
        .order_by_ambition_ordering_nulls_last(Asc)
        .order_by_direction_ordering_nulls_first(Asc)
        .order_by_action_ordering_nulls_last(Asc)
        .order_by_created_at(Asc)
        .get_all_tags()
        .await
        .map(|tags| {
            tags.into_iter()
                .map(|tag| TagVisible::from(tag))
                .collect::<Vec<_>>()
        })
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
