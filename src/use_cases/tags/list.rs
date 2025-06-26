use db_adapters::{
    tag_adapter::{TagAdapter, TagFilter, TagJoin, TagOrder, TagQuery, TagWithNames},
    Order::Asc,
};
use entities::user as user_entity;

use crate::{
    tags::types::{TagType, TagVisible},
    UseCaseError,
};

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
        .get_all_tags_with_names()
        .await
        .map(|tags| tags.iter().map(convert_to_tag_visible).collect::<Vec<_>>())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

fn convert_to_tag_visible(item: &TagWithNames) -> TagVisible {
    let (name, tag_type) = if let Some(name) = &item.name {
        (name, TagType::Plain)
    } else if let Some(name) = &item.ambition_name {
        (name, TagType::Ambition)
    } else if let Some(name) = &item.desired_state_name {
        (name, TagType::DesiredState)
    } else if let Some(name) = &item.action_name {
        (name, TagType::Action)
    } else {
        panic!("Tag without name should not exist.");
    };

    TagVisible {
        id: item.id,
        name: name.clone(),
        tag_type,
        created_at: item.created_at,
    }
}
