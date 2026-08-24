use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use common::db::Db;
use db_adapters::{
    diary_adapter::DiaryAdapter, reading_note_adapter::ReadingNoteAdapter,
    thinking_note_adapter::ThinkingNoteAdapter,
};
use entities::user as user_entity;
use use_cases::journal::{list::list_journals, types::JournalListQuery};

use crate::utils::{response_401, response_500};

#[tracing::instrument(skip(db, user))]
#[get("")]
pub async fn list_journals_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<JournalListQuery>,
) -> HttpResponse {
    match user {
        Some(user) => match list_journals(
            user.into_inner(),
            query.into_inner(),
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
