use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::diary_adapter::DiaryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::journal::diaries::delete::delete_diary;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    diary_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a diary", skip(db, user, path_param))]
#[delete("/{diary_id}")]
pub async fn delete_diary_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_diary(
                user.into_inner(),
                path_param.diary_id,
                DiaryAdapter::init(&db),
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
