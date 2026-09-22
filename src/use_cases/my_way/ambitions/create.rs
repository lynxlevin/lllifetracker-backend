use db_adapters::ambition_adapter::{AmbitionAdapter, AmbitionMutation, CreateAmbitionParams};
use entities::user as user_entity;

use crate::{
    my_way::ambitions::types::{AmbitionCreateRequest, AmbitionVisible},
    UseCaseError,
};

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        params.name.len = params.name.len(),
        params.description.is_some = params.description.is_some(),
    ),
    skip_all
)]
pub async fn create_ambition<'a>(
    user: user_entity::Model,
    params: AmbitionCreateRequest,
    ambition_adapter: AmbitionAdapter<'a>,
) -> Result<AmbitionVisible, UseCaseError> {
    ambition_adapter
        .create_with_tag(CreateAmbitionParams {
            name: params.name.clone(),
            description: params.description.clone(),
            user_id: user.id,
        })
        .await
        .map(|ambition| AmbitionVisible::from(ambition))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
