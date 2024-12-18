mod list;
mod create;
mod update;
mod delete;

use actix_web::web::{scope, ServiceConfig};

pub fn mission_memo_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/mission_memos")
            .service(list::list_mission_memos)
            .service(create::create_mission_memo)
            .service(update::update_mission_memo)
            .service(delete::delete_mission_memo),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}