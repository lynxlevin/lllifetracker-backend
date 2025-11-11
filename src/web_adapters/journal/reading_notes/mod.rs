mod create;
mod delete;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn reading_note_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/reading_notes")
            .service(create::create_reading_note_endpoint)
            .service(update::update_reading_note_endpoint)
            .service(delete::delete_reading_note_endpoint),
    );
}
