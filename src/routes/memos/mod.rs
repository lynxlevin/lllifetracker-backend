mod list;
mod create;

use actix_web::web::{scope, ServiceConfig};

pub fn memo_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/memos")
            .service(list::list_memos)
            .service(create::create_memo)
            // .service(update::update_memo)
            // .service(delete::delete_memo),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
