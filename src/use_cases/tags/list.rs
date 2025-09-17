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
        .join_desired_state()
        .join_action()
        .filter_eq_user(&user)
        .filter_out_archived()
        .order_by_ambition_created_at_nulls_last(Asc)
        .order_by_desired_state_created_at_nulls_last(Asc)
        .order_by_action_created_at_nulls_last(Asc)
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
