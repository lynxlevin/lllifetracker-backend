mod create;
mod get;
mod list;

use actix_web::web::{scope, ServiceConfig};
use create::create_action;
use get::get_action;
use list::list_actions;

pub fn action_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/actions")
            .service(list_actions)
            .service(get_action)
            .service(create_action),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
