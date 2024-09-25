use actix_web::web::scope;
use registration::{confirm_factory, register_factory, resend_email_factory};

mod get_user;
mod login;
mod logout;
mod password_change;
mod registration;

pub fn auth_routes_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        scope("/users")
            .service(login::login_user)
            .service(logout::log_out)
            .service(get_user::get_user)
            .service(
                scope("/register")
                    .service(register_factory)
                    .service(confirm_factory)
                    .service(resend_email_factory),
            ),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
