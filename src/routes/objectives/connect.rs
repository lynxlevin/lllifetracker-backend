use crate::{
    entities::user as user_entity,
    services::{
        action::Query as ActionQuery,
        objective::{Mutation as ObjectiveMutation, Query as ObjectiveQuery},
    },
    types::{self, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    objective_id: uuid::Uuid,
    action_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Connecting an action to an objective",
    skip(db, user, path_param)
)]
#[post("/{objective_id}/actions/{action_id}/connection")]
pub async fn connect_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match validate_ownership(&db, user.id, &path_param).await {
                Ok(_) => {
                    match ObjectiveMutation::connect_action(
                        &db,
                        path_param.objective_id,
                        path_param.action_id,
                    )
                    .await
                    {
                        Ok(_) => HttpResponse::Ok().json(types::SuccessResponse {
                            message: format!(
                                "Successfully connected objective: {} with action: {}",
                                path_param.objective_id, path_param.action_id
                            ),
                        }),
                        Err(e) => {
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                            HttpResponse::InternalServerError().json(types::ErrorResponse {
                                error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                            })
                        }
                    }
                }
                Err(_) => HttpResponse::NotFound().json(types::ErrorResponse {
                    error: "Objective or action with the requested ids were not found".to_string(),
                }),
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}

async fn validate_ownership(
    db: &DbConn,
    user_id: uuid::Uuid,
    path_param: &Path<PathParam>,
) -> Result<(), ()> {
    match ObjectiveQuery::find_by_id_and_user_id(db, path_param.objective_id, user_id).await {
        Ok(_) => match ActionQuery::find_by_id_and_user_id(db, path_param.action_id, user_id).await
        {
            Ok(_) => Ok(()),
            Err(e) => match e {
                DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                    CustomDbErr::NotFound => Err(()),
                },
                e => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    Err(())
                }
            },
        },
        Err(e) => match e {
            DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                CustomDbErr::NotFound => Err(()),
            },
            e => {
                tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                Err(())
            }
        },
    }
}
