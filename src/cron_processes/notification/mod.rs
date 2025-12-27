use chrono::{Datelike, NaiveTime, Timelike, Utc};
use common::{db::init_db, settings::types::Settings};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{event, instrument, Level};

mod my_way_reminder;
mod utils;

#[instrument(skip_all)]
// MYMEMO: This should panic or return Error and stop server from starting.
pub async fn run_cron_processes(settings: Settings) -> () {
    let db = init_db(&settings).await;

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

    let scheduler = match JobScheduler::new().await {
        Ok(scheduler) => scheduler,
        Err(e) => {
            event!(Level::ERROR, "{:?}", e);
            return ();
        }
    };

    if let Err(e) = scheduler
        .add(
            Job::new_async("0 0,10,20,30,40,50, * * * *", move |_, _| {
                let params = (
                    settings.clone(),
                    db.clone(),
                    weekday.clone(),
                    utc_time_rounded_by_10_minutes.clone(),
                );
                Box::pin(async move {
                    my_way_reminder::my_way_reminder(&params.0, &params.1, params.2, params.3).await
                })
            })
            .unwrap(),
        )
        .await
    {
        event!(Level::ERROR, "{:?}", e);
    };

    if let Err(e) = scheduler.start().await {
        event!(Level::ERROR, "{:?}", e);
    }

    ()
}
