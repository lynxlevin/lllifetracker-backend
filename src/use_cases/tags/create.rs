use db_adapters::tag_adapter::{CreatePlainTagParams, TagAdapter, TagMutation};
use entities::user as user_entity;

use crate::{
    tags::types::{TagCreateRequest, TagType, TagVisible},
    UseCaseError,
};

pub async fn create_plain_tag<'a>(
    user: user_entity::Model,
    params: TagCreateRequest,
    tag_adapter: TagAdapter<'a>,
) -> Result<TagVisible, UseCaseError> {
    tag_adapter
        .create_plain(CreatePlainTagParams {
            name: params.name.clone(),
            user_id: user.id,
        })
        .await
        .map(|tag| TagVisible {
            id: tag.id,
            name: tag.name.unwrap(),
            tag_type: TagType::Plain,
            created_at: tag.created_at,
        })
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
