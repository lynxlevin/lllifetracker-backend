use actix_web::web::{scope, ServiceConfig};
use password_change::{
    request_password_change, submit_password_change, verify_password_change_token,
};
use registration::{confirm_factory, register_factory, resend_email_factory};

mod get_user;
mod login;
mod logout;
mod password_change;
mod registration;
pub mod types;

pub fn auth_routes(cfg: &mut ServiceConfig) {
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
            )
            .service(
                scope("/password-change")
                    .service(request_password_change)
                    .service(verify_password_change_token)
                    .service(submit_password_change),
            ),
    );
}
