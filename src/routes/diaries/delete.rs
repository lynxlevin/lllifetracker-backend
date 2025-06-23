use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::diary_adapter::{DiaryAdapter, DiaryFilter, DiaryMutation, DiaryQuery};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    diary_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a diary", skip(db, user, path_param))]
#[delete("/{diary_id}")]
pub async fn delete_diary(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let diary = match DiaryAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.diary_id)
                .await
            {
                Ok(diary) => match diary {
                    Some(diary) => diary,
                    None => return HttpResponse::NoContent().into(),
                },
                Err(e) => return response_500(e),
            };
            match DiaryAdapter::init(&db).delete(diary).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
