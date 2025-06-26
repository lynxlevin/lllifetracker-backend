mod archive;
mod bulk_update_ordering;
mod create;
mod delete;
mod get;
mod list;
mod unarchive;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn desired_state_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/desired_states")
            .service(list::list_desired_states_endpoint)
            .service(get::get_desired_state_endpoint)
            .service(create::create_desired_state_endpoint)
            .service(bulk_update_ordering::bulk_update_desired_state_ordering_endpoint)
            .service(update::update_desired_state_endpoint)
            .service(delete::delete_desired_state_endpoint)
            .service(archive::archive_desired_state_endpoint)
            .service(unarchive::unarchive_desired_state_endpoint),
    );
}
