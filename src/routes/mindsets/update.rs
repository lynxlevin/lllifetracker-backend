use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::mindset_mutation::MindsetMutation;
use types::{
    self, CustomDbErr, MindsetUpdateRequest, MindsetVisible, INTERNAL_SERVER_ERROR_MESSAGE,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    mindset_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating an mindset", skip(db, user, req, path_param))]
#[put("/{mindset_id}")]
pub async fn update_mindset(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<MindsetUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MindsetMutation::update(
                &db,
                path_param.mindset_id,
                user.id,
                req.name.clone(),
                req.description.clone(),
            )
            .await
            {
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
