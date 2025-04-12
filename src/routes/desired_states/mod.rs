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
            .service(list::list_desired_states)
            .service(get::get_desired_state)
            .service(create::create_desired_state)
            .service(bulk_update_ordering::bulk_update_desired_state_ordering)
            .service(update::update_desired_state)
            .service(delete::delete_desired_state)
            .service(archive::archive_desired_state)
            .service(unarchive::unarchive_desired_state),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
