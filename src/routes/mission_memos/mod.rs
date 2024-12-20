mod archive;
mod create;
mod delete;
mod list;
mod mark_accomplished;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn mission_memo_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/mission_memos")
            .service(list::list_mission_memos)
            .service(create::create_mission_memo)
            .service(update::update_mission_memo)
            .service(delete::delete_mission_memo)
            .service(archive::archive_mission_memo)
            .service(mark_accomplished::mark_accomplished_mission_memo),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
