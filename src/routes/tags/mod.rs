mod create;
mod delete;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn tag_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/tags")
            .service(create::create_plain_tag)
            .service(delete::delete_plain_tag)
            .service(list::list_tags)
            .service(update::update_plain_tag),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
