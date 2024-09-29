mod create;
mod delete;
mod get;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};
use create::create_objective;
use delete::delete_objective;
use get::get_objective;
use list::list_objectives;
use update::update_objective;

pub fn objective_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/objectives")
            .service(list_objectives)
            .service(get_objective)
            .service(create_objective)
            .service(update_objective)
            .service(delete_objective),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
