use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use db_adapters::thinking_note_adapter::ThinkingNoteAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::journal::thinking_notes::{list::list_thinking_notes, types::ThinkingNoteListQuery};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing user's thinking notes.", skip(db, user))]
#[get("")]
pub async fn list_thinking_notes_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<ThinkingNoteListQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_thinking_notes(
                user.into_inner(),
                query.into_inner(),
                ThinkingNoteAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
