use uuid::Uuid;

use crate::{my_way::ambitions::types::AmbitionVisible, UseCaseError};
use db_adapters::ambition_adapter::{
    AmbitionAdapter, AmbitionFilter, AmbitionMutation, AmbitionQuery,
};
use entities::user as user_entity;

pub async fn archive_ambition<'a>(
    user: user_entity::Model,
    ambition_id: Uuid,
    ambition_adapter: AmbitionAdapter<'a>,
) -> Result<AmbitionVisible, UseCaseError> {
    let ambition = ambition_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(ambition_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Ambition with this id was not found".to_string(),
        ))?;

    ambition_adapter
        .archive(ambition)
        .await
        .map(|ambition| AmbitionVisible::from(ambition))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
