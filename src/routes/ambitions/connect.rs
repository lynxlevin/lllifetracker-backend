use crate::{
    entities::user as user_entity,
    services::{
        ambition::{Mutation as AmbitionMutation, Query as AmbitionQuery},
        objective::Query as ObjectiveQuery,
    },
    startup::AppState,
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
    ambition_id: uuid::Uuid,
    objective_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Connecting an objective to an ambition",
    skip(data, user, path_param)
)]
#[post("/{ambition_id}/objectives/{objective_id}/connection")]
pub async fn connect_objective(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match validate_ownership(&data.conn, user.id, &path_param).await {
                Ok(_) => {
                    match AmbitionMutation::connect_objective(
                        &data.conn,
                        path_param.ambition_id,
                        path_param.objective_id,
                    )
                    .await
                    {
                        Ok(_) => HttpResponse::Ok().json(types::SuccessResponse {
                            message: format!(
                                "Successfully connected ambition: {} with objective: {}",
                                path_param.ambition_id, path_param.objective_id
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
                    error: "Ambition or objective with the requested ids were not found"
                        .to_string(),
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
    match AmbitionQuery::find_by_id_and_user_id(db, path_param.ambition_id, user_id).await {
        Ok(_) => match ObjectiveQuery::find_by_id_and_user_id(db, path_param.objective_id, user_id)
            .await
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
