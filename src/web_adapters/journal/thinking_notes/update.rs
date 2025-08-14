use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::thinking_note_adapter::ThinkingNoteAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    journal::thinking_notes::{types::ThinkingNoteUpdateRequest, update::update_thinking_note},
    UseCaseError,
};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    thinking_note_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating a thinking note", skip(db, user, req, path_param))]
#[put("/{thinking_note_id}")]
pub async fn update_thinking_note_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ThinkingNoteUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_thinking_note(
                user.into_inner(),
                req.into_inner(),
                path_param.thinking_note_id,
                ThinkingNoteAdapter::init(&db),
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
