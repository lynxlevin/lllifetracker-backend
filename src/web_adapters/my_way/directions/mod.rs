mod archive;
mod bulk_update_ordering;
mod create;
mod delete;
mod get;
mod list;
mod unarchive;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn direction_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/directions")
            .service(list::list_directions_endpoint)
            .service(get::get_direction_endpoint)
            .service(create::create_direction_endpoint)
            .service(bulk_update_ordering::bulk_update_direction_ordering_endpoint)
            .service(update::update_direction_endpoint)
            .service(delete::delete_direction_endpoint)
            .service(archive::archive_direction_endpoint)
            .service(unarchive::unarchive_direction_endpoint),
    );
}
