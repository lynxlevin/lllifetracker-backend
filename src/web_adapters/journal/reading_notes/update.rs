use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::reading_note_adapter::ReadingNoteAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    journal::reading_notes::{types::ReadingNoteUpdateRequest, update::update_reading_note},
    UseCaseError,
};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    reading_note_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating a reading note", skip(db, user, req, path_param))]
#[put("/{reading_note_id}")]
pub async fn update_reading_note_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ReadingNoteUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_reading_note(
                user.into_inner(),
                req.into_inner(),
                path_param.reading_note_id,
                ReadingNoteAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
