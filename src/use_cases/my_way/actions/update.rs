use domain_services::action_service::{
    ActionService, ActionServiceError, ActionServiceMutation, ActionUpdateParams,
};
use thiserror::Error;
use uuid::Uuid;

use crate::my_way::actions::types::{ActionUpdateRequest, ActionVisible};
use entities::user as user_entity;

#[derive(Debug, Error)]
pub enum UpdateActionError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionServiceError> for UpdateActionError {
    fn from(e: ActionServiceError) -> Self {
        match e {
            ActionServiceError::ActionNotFound() => Self::NotFound(e.to_string()),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        action_id = action_id.to_string(),
        params.name.len = params.name.len(),
        params.discipline.is_some = params.discipline.is_some(),
        params.memo.is_some = params.memo.is_some(),
        params.color = params.color,
    ),
    skip_all
)]
pub async fn update_action<'a>(
    user: user_entity::Model,
    params: ActionUpdateRequest,
    action_id: Uuid,
    action_service: ActionService<'a>,
) -> Result<ActionVisible, UpdateActionError> {
    let params = _parse_params(params)?;

    let action = action_service
        .update(ActionUpdateParams {
            name: params.name,
            discipline: params.discipline,
            memo: params.memo,
            color: params.color,
            user_id: user.id,
            action_id,
        })
        .await?;

    Ok(ActionVisible::from(action))
}

fn _parse_params(params: ActionUpdateRequest) -> Result<ActionUpdateRequest, UpdateActionError> {
    if params.color.len() != 7 {
        return Err(UpdateActionError::BadRequest(
            "color must be 7 characters long.".to_string(),
        ));
    }
    if !params.color.starts_with('#') {
        return Err(UpdateActionError::BadRequest(
            "color must be hex color code.".to_string(),
        ));
    }
    for c in params.color.split_at(1).1.chars() {
        if !c.is_ascii_hexdigit() {
            return Err(UpdateActionError::BadRequest(
                "color must be hex color code.".to_string(),
            ));
        }
    }

    Ok(params)
}
