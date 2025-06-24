mod archive;
mod bulk_update_ordering;
mod convert_track_type;
mod create;
mod delete;
mod get;
mod list;
mod unarchive;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn action_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/actions")
            .service(list::list_actions_endpoint)
            .service(get::get_action_endpoint)
            .service(create::create_action_endpoint)
            .service(bulk_update_ordering::bulk_update_action_ordering_endpoint)
            .service(update::update_action_endpoint)
            .service(convert_track_type::convert_action_track_type_endpoint)
            .service(delete::delete_action_endpoint)
            .service(archive::archive_action_endpoint)
            .service(unarchive::unarchive_action_endpoint),
    );
}
