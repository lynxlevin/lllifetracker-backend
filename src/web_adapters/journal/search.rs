use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::{
    diary_service::DiaryService, reading_note_service::ReadingNoteService,
    thinking_note_service::ThinkingNoteService,
};
use entities::user as user_entity;
use use_cases::journal::{search::search_journals, types::JournalSearchRequest};

use common::errors::error_responses::{response_401, response_500};

#[post("search")]
pub async fn search_journals_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    params: Json<JournalSearchRequest>,
) -> HttpResponse {
    match user {
        Some(user) => match search_journals(
            user.into_inner(),
            params.into_inner(),
            DiaryService::init(&db),
            ReadingNoteService::init(&db),
            ThinkingNoteService::init(&db),
        )
        .await
        {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(e) => response_500(e),
        },
        None => response_401(),
    }
}
