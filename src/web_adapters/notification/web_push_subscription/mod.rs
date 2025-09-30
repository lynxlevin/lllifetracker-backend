mod create;

use actix_web::web::{scope, ServiceConfig};

pub fn web_push_subscription_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/web_push_subscription").service(create::create_web_push_subscription_endpoint),
    );
}
