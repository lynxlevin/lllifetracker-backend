mod create;
mod delete;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn thinking_note_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/thinking_notes")
            .service(create::create_thinking_note_endpoint)
            .service(delete::delete_thinking_note_endpoint)
            .service(update::update_thinking_note_endpoint),
    );
}
