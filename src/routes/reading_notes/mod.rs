mod create;
mod delete;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn reading_note_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/reading_notes")
            .service(list::list_reading_notes)
            .service(create::create_reading_note)
            .service(update::update_reading_note)
            .service(delete::delete_reading_note),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
