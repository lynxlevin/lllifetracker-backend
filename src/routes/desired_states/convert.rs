use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr, TransactionError};
use services::desired_state_mutation::DesiredStateMutation;
use types::{CustomDbErr, DesiredStateConvertRequest, MindsetVisible};

use crate::utils::{response_401, response_404, response_500};

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
                Err(e) => match &e {
                    TransactionError::Transaction(DbErr::Custom(e)) => {
                        match e.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                response_404("DesiredState with this id was not found")
                            }
                            _ => response_500(e),
                        }
                    }
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
