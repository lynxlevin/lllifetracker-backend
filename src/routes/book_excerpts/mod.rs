mod create;
mod delete;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn book_excerpt_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/book_excerpts")
            .service(list::list_book_excerpts)
            .service(create::create_book_excerpt)
            .service(update::update_book_excerpt)
            .service(delete::delete_book_excerpt),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
