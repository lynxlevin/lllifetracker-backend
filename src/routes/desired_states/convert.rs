use ::types::{self, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr, TransactionError};
use services::desired_state_mutation::DesiredStateMutation;
use types::{DesiredStateConvertRequest, MindsetVisible};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(name = "Converting an desired_state", skip(db, user, path_param))]
#[put("/{desired_state_id}/convert")]
pub async fn convert_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateConvertRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateMutation::convert(
                &db,
                path_param.desired_state_id,
                user.id,
                req.convert_to.clone(),
            )
            .await
            {
                Ok(mindset) => {
                    let res: MindsetVisible = mindset.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => {
                    match &e {
                        TransactionError::Transaction(DbErr::Custom(e)) => {
                            match e.parse::<CustomDbErr>().unwrap() {
                                CustomDbErr::NotFound => {
                                    return HttpResponse::NotFound().json(types::ErrorResponse {
                                        error: "DesiredState with this id was not found"
                                            .to_string(),
                                    })
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
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}
