mod archive;
mod bulk_update_ordering;
mod connect;
mod create;
mod delete;
mod disconnect;
mod get;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn ambition_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/ambitions")
            .service(list::list_ambitions)
            .service(get::get_ambition)
            .service(create::create_ambition)
            .service(bulk_update_ordering::bulk_update_ambition_ordering)
            .service(update::update_ambition)
            .service(delete::delete_ambition)
            .service(archive::archive_ambition)
            .service(connect::connect_desired_state)
            .service(disconnect::disconnect_desired_state),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
