pub mod diaries;
mod list;
pub mod reading_notes;
mod search;
pub mod thinking_notes;

use actix_web::web::{scope, ServiceConfig};

pub fn journal_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/journals")
            .service(list::list_journals_endpoint)
            .service(search::search_journals_endpoint),
    );
}
