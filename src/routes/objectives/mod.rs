mod connect;
mod create;
mod delete;
mod disconnect;
mod get;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn objective_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/objectives")
            .service(list::list_objectives)
            .service(get::get_objective)
            .service(create::create_objective)
            .service(update::update_objective)
            .service(delete::delete_objective)
            .service(connect::connect_action)
            .service(disconnect::disconnect_action),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
