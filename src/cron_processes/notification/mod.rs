use chrono::{Datelike, NaiveTime, Timelike, Utc};
use common::{db::init_db, settings::types::Settings};
use tracing::{event, instrument, Level};

mod my_way_reminder;
mod utils;

#[instrument(skip_all)]
pub async fn run_cron_processes(settings: &Settings) -> () {
    let db = init_db(settings).await;

    let now = Utc::now();
    let weekday = now.weekday();
    let utc_time_rounded_by_10_minutes =
        match NaiveTime::from_hms_opt(now.hour(), (now.minute() + 5) / 10 * 10, 0) {
            Some(time) => time,
            None => {
                event!(Level::ERROR, "Error on parsing utc_time. now: {}", now);
                return ();
            }
        };
    my_way_reminder::my_way_reminder(
        settings,
        &db,
        weekday.clone(),
        utc_time_rounded_by_10_minutes.clone(),
    )
    .await;
}
