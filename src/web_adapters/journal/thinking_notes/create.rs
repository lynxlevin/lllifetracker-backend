use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::thinking_note_adapter::ThinkingNoteAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    journal::thinking_notes::{create::create_thinking_note, types::ThinkingNoteCreateRequest},
    UseCaseError,
};

use crate::utils::{response_401, response_404, response_500};

#[tracing::instrument(name = "Creating a thinking note", skip(db, user))]
#[post("")]
pub async fn create_thinking_note_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ThinkingNoteCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_thinking_note(
                user.into_inner(),
                req.into_inner(),
                ThinkingNoteAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
