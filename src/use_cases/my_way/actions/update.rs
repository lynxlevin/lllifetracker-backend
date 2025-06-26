use uuid::Uuid;

use crate::{
    my_way::actions::types::{ActionUpdateRequest, ActionVisible},
    UseCaseError,
};
use db_adapters::action_adapter::{
    ActionAdapter, ActionFilter, ActionMutation, ActionQuery, UpdateActionParams,
};
use entities::user as user_entity;

pub async fn update_action<'a>(
    user: user_entity::Model,
    params: ActionUpdateRequest,
    action_id: Uuid,
    action_adapter: ActionAdapter<'a>,
) -> Result<ActionVisible, UseCaseError> {
    if let Err(message) = _validate_params(&params) {
        return Err(UseCaseError::BadRequest(message));
    }

    let action = action_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(action_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Action with this id was not found".to_string(),
        ))?;

    action_adapter
        .update(
            action,
            UpdateActionParams {
                name: params.name.clone(),
                description: params.description.clone(),
                trackable: params.trackable,
                color: params.color.clone(),
            },
        )
        .await
        .map(|action| ActionVisible::from(action))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

fn _validate_params(params: &ActionUpdateRequest) -> Result<(), String> {
    if let Some(color) = &params.color {
        if color.len() != 7 {
            return Err("color must be 7 characters long.".to_string());
        }
        if !color.starts_with('#') {
            return Err("color must be hex color code.".to_string());
        }
        for c in color.split_at(1).1.chars() {
            if !c.is_ascii_hexdigit() {
                return Err("color must be hex color code.".to_string());
            }
        }
    }

    Ok(())
}
