mod list;

use actix_web::web::{scope, ServiceConfig};
use list::list_actions;

pub fn action_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/actions").service(list_actions),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
