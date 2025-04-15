use types::ActionUpdateRequest;
use ::types::{self, ActionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::action_mutation::ActionMutation;

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
                        Err(e) => {
                            match &e {
                                DbErr::Custom(message) => {
                                    match message.parse::<CustomDbErr>().unwrap() {
                                        CustomDbErr::NotFound => {
                                            return HttpResponse::NotFound().json(
                                                types::ErrorResponse {
                                                    error: "Action with this id was not found"
                                                        .to_string(),
                                                },
                                            )
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                            HttpResponse::InternalServerError().json(types::ErrorResponse {
                                error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                            })
                        }
                    }
                }
                Err(e) => HttpResponse::BadRequest().json(types::ErrorResponse { error: e }),
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
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
