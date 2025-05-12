use ::types::{self, ActionVisible, CustomDbErr};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::action_mutation::ActionMutation;
use types::ActionUpdateRequest;

use crate::utils::{response_400, response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating an action", skip(db, user, req, path_param))]
#[put("/{action_id}")]
pub async fn update_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match _validate_request_body(&req) {
                Ok(_) => {
                    match ActionMutation::update(
                        &db,
                        path_param.action_id,
                        user.id,
                        req.name.clone(),
                        req.description.clone(),
                        req.trackable,
                        req.color.clone(),
                    )
                    .await
                    {
                        Ok(action) => {
                            let res: ActionVisible = action.into();
                            HttpResponse::Ok().json(res)
                        }
                        Err(e) => match &e {
                            DbErr::Custom(message) => {
                                match message.parse::<CustomDbErr>().unwrap() {
                                    CustomDbErr::NotFound => {
                                        response_404("Action with this id was not found")
                                    }
                                    _ => response_500(e),
                                }
                            }
                            _ => response_500(e),
                        },
                    }
                }
                Err(e) => response_400(&e),
            }
        }
        None => response_401(),
    }
}

fn _validate_request_body(req: &ActionUpdateRequest) -> Result<(), String> {
    if let Some(color) = &req.color {
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
