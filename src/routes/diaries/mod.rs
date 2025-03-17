mod list;
// mod create;
// mod update;
// mod delete;

use actix_web::web::{scope, ServiceConfig};

pub fn diary_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/diaries")
            .service(list::list_diaries)
            // .service(create::create_diary)
            // .service(update::update_diary)
            // .service(delete::delete_diary),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
