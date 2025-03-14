mod archive;
mod create;
mod delete;
mod list;
mod mark_accomplished;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn challenge_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/challenges")
            .service(list::list_challenges)
            .service(create::create_challenge)
            .service(update::update_challenge)
            .service(delete::delete_challenge)
            .service(archive::archive_challenge)
            .service(mark_accomplished::mark_accomplished_challenge),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
