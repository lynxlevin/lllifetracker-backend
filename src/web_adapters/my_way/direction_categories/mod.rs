mod bulk_update_ordering;
mod create;
mod delete;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn direction_category_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/direction_categories")
            .service(list::list_direction_categories_endpoint)
            .service(bulk_update_ordering::bulk_update_direction_category_ordering_endpoint)
            .service(create::create_direction_category_endpoint)
            .service(update::update_direction_category_endpoint)
            .service(delete::delete_direction_category_endpoint),
    );
}
