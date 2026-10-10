use crate::my_way::actions::types::{ActionCreateRequest, ActionVisible};
use domain_services::action_service::{
    ActionCreateParams, ActionService, ActionServiceError, ActionServiceMutation,
};
use entities::user as user_entity;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateActionError {
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionServiceError> for CreateActionError {
    fn from(e: ActionServiceError) -> Self {
        match e {
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        params.name.len = params.name.len(),
        params.discipline.is_some = params.discipline.is_some(),
        params.memo.is_some = params.memo.is_some(),
        params.track_type = params.track_type.to_string(),
    ),
    skip_all
)]
pub async fn create_action<'a>(
    user: user_entity::Model,
    params: ActionCreateRequest,
    action_service: ActionService<'a>,
) -> Result<ActionVisible, CreateActionError> {
    let action = action_service
        .create(ActionCreateParams {
            name: params.name.clone(),
            discipline: params.discipline.clone(),
            memo: params.memo.clone(),
            color: params.color.clone(),
            track_type: params.track_type.clone(),
            user_id: user.id,
        })
        .await?;

    Ok(ActionVisible::from(action))
}
