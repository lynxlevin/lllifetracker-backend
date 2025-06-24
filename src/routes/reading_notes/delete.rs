use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::reading_note_adapter::{
    ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteMutation, ReadingNoteQuery,
};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    reading_note_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a reading note", skip(db, user, path_param))]
#[delete("/{reading_note_id}")]
pub async fn delete_reading_note(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let reading_note = match ReadingNoteAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.reading_note_id)
                .await
            {
                Ok(res) => match res {
                    Some(reading_note) => reading_note,
                    None => return HttpResponse::NoContent().finish(),
                },
                Err(e) => return response_500(e),
            };
            match ReadingNoteAdapter::init(&db).delete(reading_note).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
