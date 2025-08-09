mod create;

use actix_web::web::{scope, ServiceConfig};

pub fn action_goal_routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/action_goals").service(create::create_action_goal_endpoint));
}
