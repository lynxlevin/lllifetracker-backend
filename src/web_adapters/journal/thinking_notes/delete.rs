use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::thinking_note_adapter::ThinkingNoteAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::journal::thinking_notes::delete::delete_thinking_note;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    thinking_note_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a thinking note", skip(db, user, path_param))]
#[delete("/{thinking_note_id}")]
pub async fn delete_thinking_note_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_thinking_note(
                user.into_inner(),
                path_param.thinking_note_id,
                ThinkingNoteAdapter::init(&db),
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
