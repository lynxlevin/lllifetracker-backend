mod bulk_update_ordering;
mod create;
mod delete;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn desired_state_category_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/desired_state_categories")
            .service(list::list_desired_state_categories_endpoint)
            .service(bulk_update_ordering::bulk_update_desired_state_category_ordering_endpoint)
            .service(create::create_desired_state_category_endpoint)
            .service(update::update_desired_state_category_endpoint)
            .service(delete::delete_desired_state_category_endpoint),
    );
}
