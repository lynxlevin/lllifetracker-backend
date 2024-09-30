use crate::{
    entities::user as user_entity,
    services::action::Mutation as ActionMutation,
    startup::AppState,
    types::{self, ActionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::DbErr;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
}

#[tracing::instrument(name = "Updating an action", skip(data, user, req, path_param))]
#[put("/{action_id}")]
pub async fn update_action(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::update(
                &data.conn,
                path_param.action_id,
                user.id,
                req.name.clone(),
            )
            .await
            {
                Ok(action) => HttpResponse::Ok().json(ActionVisible {
                    id: action.id,
                    name: action.name,
                    created_at: action.created_at,
                    updated_at: action.updated_at,
                }),
                Err(e) => match e {
                    DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "Action with this id was not found".to_string(),
                            })
                        }
                    },
                    e => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                        HttpResponse::InternalServerError().json(types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                },
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}
