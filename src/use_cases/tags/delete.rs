use db_adapters::tag_adapter::{TagAdapter, TagFilter, TagMutation, TagQuery};
use entities::{tag, user as user_entity};
use uuid::Uuid;

use crate::UseCaseError;

pub async fn delete_plain_tag<'a>(
    user: user_entity::Model,
    tag_id: Uuid,
    tag_adapter: TagAdapter<'a>,
) -> Result<(), UseCaseError> {
    let tag = match tag_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(tag_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(tag) => tag,
        None => return Ok(()),
    };

    if !_is_plain_tag(&tag) {
        return Err(UseCaseError::BadRequest(
            "Tag to delete must be a plain tag.".to_string(),
        ));
    };
    tag_adapter
        .delete(tag)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

fn _is_plain_tag(tag: &tag::Model) -> bool {
    return tag.name.is_some()
        && tag.ambition_id.is_none()
        && tag.desired_state_id.is_none()
        && tag.action_id.is_none();
}
