mod archive;
mod bulk_update_ordering;
mod create;
mod delete;
mod get;
mod list;
mod unarchive;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn mindset_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/mindsets")
            .service(list::list_mindsets)
            .service(get::get_mindset)
            .service(create::create_mindset)
            .service(bulk_update_ordering::bulk_update_mindset_ordering)
            .service(update::update_mindset)
            .service(delete::delete_mindset)
            .service(archive::archive_mindset)
            .service(unarchive::unarchive_mindset),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
