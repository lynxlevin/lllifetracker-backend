mod create;
mod list;

use actix_web::web::{scope, ServiceConfig};

pub fn tag_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/tags")
            .service(create::create_tag)
            .service(list::list_tags),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
