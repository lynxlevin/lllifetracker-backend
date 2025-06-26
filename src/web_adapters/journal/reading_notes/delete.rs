use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::reading_note_adapter::ReadingNoteAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::journal::reading_notes::delete::delete_reading_note;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    reading_note_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a reading note", skip(db, user, path_param))]
#[delete("/{reading_note_id}")]
pub async fn delete_reading_note_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_reading_note(
                user.into_inner(),
                path_param.reading_note_id,
                ReadingNoteAdapter::init(&db),
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
