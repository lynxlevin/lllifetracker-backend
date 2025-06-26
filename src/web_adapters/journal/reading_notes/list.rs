use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::reading_note_adapter::ReadingNoteAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::journal::reading_notes::list::list_reading_notes;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing user's reading notes.", skip(db, user))]
#[get("")]
pub async fn list_reading_notes_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_reading_notes(user.into_inner(), ReadingNoteAdapter::init(&db)).await {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
