use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::mindset_mutation::MindsetMutation;
use types::{CustomDbErr, MindsetUpdateRequest, MindsetVisible};

use crate::utils::{response_401, response_404, response_500};

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
                Err(e) => match &e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => response_404("Mindset with this id was not found"),
                        _ => response_500(e),
                    },
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
