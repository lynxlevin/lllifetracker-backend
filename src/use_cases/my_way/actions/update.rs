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
    let params = _parse_params(params)?;

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
                discipline: params.discipline.clone(),
                memo: params.memo.clone(),
                color: params.color.clone(),
            },
        )
        .await
        .map(|action| ActionVisible::from(action))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

fn _parse_params(params: ActionUpdateRequest) -> Result<ActionUpdateRequest, UseCaseError> {
    if let Some(color) = &params.color {
        if color.len() != 7 {
            return Err(UseCaseError::BadRequest(
                "color must be 7 characters long.".to_string(),
            ));
        }
        if !color.starts_with('#') {
            return Err(UseCaseError::BadRequest(
                "color must be hex color code.".to_string(),
            ));
        }
        for c in color.split_at(1).1.chars() {
            if !c.is_ascii_hexdigit() {
                return Err(UseCaseError::BadRequest(
                    "color must be hex color code.".to_string(),
                ));
            }
        }
    }

    Ok(params)
}
