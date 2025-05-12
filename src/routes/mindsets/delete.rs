use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::mindset_mutation::MindsetMutation;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    mindset_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an mindset", skip(db, user, path_param))]
#[delete("/{mindset_id}")]
pub async fn delete_mindset(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MindsetMutation::delete(&db, path_param.mindset_id, user.id).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
