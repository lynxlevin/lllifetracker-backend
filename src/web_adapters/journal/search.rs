use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::{
    diary_adapter::DiaryAdapter, reading_note_adapter::ReadingNoteAdapter,
    thinking_note_adapter::ThinkingNoteAdapter,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::journal::{search::search_journals, types::JournalSearchRequest};

use crate::utils::{response_401, response_500};

#[tracing::instrument(skip(db, user))]
#[post("search")]
pub async fn search_journals_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    params: Json<JournalSearchRequest>,
) -> HttpResponse {
    match user {
        Some(user) => match search_journals(
            user.into_inner(),
            params.into_inner(),
            DiaryAdapter::init(&db),
            ReadingNoteAdapter::init(&db),
            ThinkingNoteAdapter::init(&db),
        )
        .await
        {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(e) => response_500(e),
        },
        None => response_401(),
    }
}
