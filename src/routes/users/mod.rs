mod confirm_registration;
mod get_user;
mod login;
mod logout;
mod register;
mod resend_email;

pub fn auth_routes_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/users")
            .service(register::register_user)
            .service(confirm_registration::confirm)
            .service(resend_email::resend_email)
            .service(login::login_user)
            .service(logout::log_out)
            .service(get_user::get_user),
        // MYMEMO: Can restrict AuthenticateUser this way.
        // .service(
        //     actix_web::web::scope("")
        //         .wrap(AuthenticateUser)
        //         .service(get_user::get_user),
        // ),
    );
}
