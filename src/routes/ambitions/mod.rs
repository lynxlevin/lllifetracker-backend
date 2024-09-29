mod create;
mod delete;
mod get;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};
use create::create_ambition;
use delete::delete_ambition;
use get::get_ambition;
use list::list_ambitions;
use update::update_ambition;

pub fn ambition_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/ambitions")
            .service(list_ambitions)
            .service(get_ambition)
            .service(create_ambition)
            .service(update_ambition)
            .service(delete_ambition),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
