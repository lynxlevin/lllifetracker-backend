mod create;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn desired_state_category_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/desired_state_categories")
            .service(list::list_desired_state_categories)
            .service(create::create_desired_state_category)
            .service(update::update_desired_state_category),
    );
}
