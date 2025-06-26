use db_adapters::tag_adapter::{
    TagAdapter, TagFilter, TagMutation, TagQuery, UpdatePlainTagParams,
};
use entities::{tag, user as user_entity};
use uuid::Uuid;

use crate::{
    tags::types::{TagType, TagUpdateRequest, TagVisible},
    UseCaseError,
};

pub async fn update_plain_tag<'a>(
    user: user_entity::Model,
    params: TagUpdateRequest,
    tag_id: Uuid,
    tag_adapter: TagAdapter<'a>,
) -> Result<TagVisible, UseCaseError> {
    let tag = tag_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(tag_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Tag with this id was not found".to_string(),
        ))?;

    if !_is_plain_tag(&tag) {
        return Err(UseCaseError::BadRequest(
            "Tag to update must be a plain tag.".to_string(),
        ));
    };

    tag_adapter
        .update_plain(
            tag,
            UpdatePlainTagParams {
                name: params.name.clone(),
            },
        )
        .await
        .map(|tag| TagVisible {
            id: tag.id,
            name: tag.name.unwrap(),
            tag_type: TagType::Plain,
            created_at: tag.created_at,
        })
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

fn _is_plain_tag(tag: &tag::Model) -> bool {
    return tag.name.is_some()
        && tag.ambition_id.is_none()
        && tag.desired_state_id.is_none()
        && tag.action_id.is_none();
}
