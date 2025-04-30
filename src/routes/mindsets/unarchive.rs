use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::mindset_mutation::MindsetMutation;
use types::{self, CustomDbErr, MindsetVisible, INTERNAL_SERVER_ERROR_MESSAGE};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    mindset_id: uuid::Uuid,
}

#[tracing::instrument(name = "Restoring an mindset from archive", skip(db, user, path_param))]
#[put("/{mindset_id}/unarchive")]
pub async fn unarchive_mindset(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MindsetMutation::unarchive(&db, path_param.mindset_id, user.id).await {
                Ok(mindset) => {
                    let res: MindsetVisible = mindset.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => {
                    match &e {
                        DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                return HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "Mindset with this id was not found".to_string(),
                                })
                            }
                            _ => {}
                        },
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
