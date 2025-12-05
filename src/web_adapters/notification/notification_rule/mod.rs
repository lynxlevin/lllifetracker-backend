pub mod delete;
pub mod list;

use actix_web::web::{scope, ServiceConfig};

pub fn notification_rule_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/notification_rules")
            .service(list::list_notification_rules_endpoint)
            .service(delete::delete_notification_rules_endpoint),
    );
}
