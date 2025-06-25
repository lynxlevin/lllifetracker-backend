mod archive;
mod bulk_update_ordering;
mod create;
mod delete;
mod get;
mod list;
mod unarchive;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn ambition_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/ambitions")
            .service(list::list_ambitions_endpoint)
            .service(get::get_ambition_endpoint)
            .service(create::create_ambition_endpoint)
            .service(bulk_update_ordering::bulk_update_ambition_ordering_endpoint)
            .service(update::update_ambition_endpoint)
            .service(delete::delete_ambition_endpoint)
            .service(archive::archive_ambition_endpoint)
            .service(unarchive::unarchive_ambition_endpoint),
    );
}
