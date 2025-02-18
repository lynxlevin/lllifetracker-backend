mod archive;
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
            .service(update::update_ambition)
            .service(delete::delete_ambition)
            .service(archive::archive_ambition)
            .service(connect::connect_objective)
            .service(disconnect::disconnect_objective),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
