mod create;
mod delete;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn diary_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/diaries")
            .service(create::create_diary_endpoint)
            .service(update::update_diary_endpoint)
            .service(delete::delete_diary_endpoint),
    );
}
