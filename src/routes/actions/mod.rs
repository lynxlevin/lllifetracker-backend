mod create;
mod delete;
mod get;
mod list;
mod update;
mod archive;
mod bulk_update_ordering;

use actix_web::web::{scope, ServiceConfig};

pub fn action_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/actions")
            .service(list::list_actions)
            .service(get::get_action)
            .service(create::create_action)
            .service(update::update_action)
            .service(delete::delete_action)
            .service(archive::archive_action)
            .service(bulk_update_ordering::bulk_update_action_ordering),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
