mod confirm;
mod register;
mod resend_email;

pub use confirm::confirm as confirm_factory;
pub use register::register as register_factory;
pub use resend_email::resend_email as resend_email_factory;
