mod create;
mod delete;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn action_track_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/action_tracks")
            .service(list::list_action_tracks)
            .service(create::create_action_track)
            .service(update::update_action_track)
            .service(delete::delete_action_track)
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
