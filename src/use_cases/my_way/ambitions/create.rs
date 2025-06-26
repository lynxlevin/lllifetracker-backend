use db_adapters::ambition_adapter::{AmbitionAdapter, AmbitionMutation, CreateAmbitionParams};
use entities::user as user_entity;

use crate::{
    my_way::ambitions::types::{AmbitionCreateRequest, AmbitionVisible},
    UseCaseError,
};

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
