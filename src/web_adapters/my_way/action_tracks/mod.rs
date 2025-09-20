mod aggregation;
mod aggregation_daily;
mod create;
mod delete;
mod list;
mod update;

use actix_web::web::{scope, ServiceConfig};

pub fn action_track_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/action_tracks")
            .service(list::list_action_tracks_endpoint)
            .service(create::create_action_track_endpoint)
            .service(update::update_action_track_endpoint)
            .service(delete::delete_action_track_endpoint)
            .service(aggregation::aggregate_action_tracks_endpoint)
            .service(aggregation_daily::aggregate_daily_action_tracks_endpoint),
    );
}
