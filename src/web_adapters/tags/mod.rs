mod create;
mod delete;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn tag_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/tags")
            .service(create::create_plain_tag_endpoint)
            .service(delete::delete_plain_tag_endpoint)
            .service(list::list_tags_endpoint)
            .service(update::update_plain_tag_endpoint),
    );
}
