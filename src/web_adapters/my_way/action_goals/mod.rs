mod set_new;

use actix_web::web::{scope, ServiceConfig};

pub fn action_goal_routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/action_goals").service(set_new::set_new_action_goal_endpoint));
}
