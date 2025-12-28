use chrono::{DateTime, Datelike, Duration, NaiveTime, Timelike, Utc, Weekday};
use common::{db::init_db, settings::types::Settings};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{event, instrument, Level};

mod my_way_reminder;
mod utils;

#[instrument(skip_all)]
pub async fn run_cron_processes(settings: Settings) -> Result<(), ()> {
    let db = init_db(&settings).await;

    let scheduler = match JobScheduler::new().await {
        Ok(scheduler) => scheduler,
        Err(e) => {
            event!(Level::ERROR, "{:?}", e);
            return Err(());
        }
    };

    let my_way_reminder_job = match Job::new_async("0 0,10,20,30,40,50, * * * *", move |_, _| {
        let params = (settings.clone(), db.clone());
        Box::pin(async move {
            let (weekday, utc_time_rounded_by_10_minutes) = match get_parsed_time(Utc::now()) {
                Some(parsed_now) => parsed_now,
                None => {
                    event!(Level::ERROR, "Error on parsing utc_time.");
                    return ();
                }
            };
            my_way_reminder::my_way_reminder(
                &params.0,
                &params.1,
                weekday,
                utc_time_rounded_by_10_minutes,
            )
            .await
        })
    }) {
        Ok(job) => job,
        Err(e) => {
            event!(Level::ERROR, "{:?}", e);
            return Err(());
        }
    };
    if let Err(e) = scheduler.add(my_way_reminder_job).await {
        event!(Level::ERROR, "{:?}", e);
        return Err(());
    };

    if let Err(e) = scheduler.start().await {
        event!(Level::ERROR, "{:?}", e);
        return Err(());
    }

    Ok(())
}

fn get_parsed_time(time: DateTime<Utc>) -> Option<(Weekday, NaiveTime)> {
    let five_minutes_ahead = time + Duration::minutes(5);
    let weekday = five_minutes_ahead.weekday();
    let utc_time_rounded_by_10_minutes = match NaiveTime::from_hms_opt(
        five_minutes_ahead.hour(),
        five_minutes_ahead.minute() / 10 * 10,
        0,
    ) {
        Some(time) => time,
        None => {
            return None;
        }
    };
    Some((weekday, utc_time_rounded_by_10_minutes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_parsed_time() {
        let time = DateTime::parse_from_rfc3339("2025-12-28T00:00:00Z")
            .unwrap()
            .to_utc();
        let res = get_parsed_time(time);
        assert_eq!(
            res,
            Some((Weekday::Sun, NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
        );
    }

    #[test]
    fn test_get_parsed_time_55_minutes_to_0() {
        let time = DateTime::parse_from_rfc3339("2025-12-28T23:55:00Z")
            .unwrap()
            .to_utc();
        let res = get_parsed_time(time);
        assert_eq!(
            res,
            Some((Weekday::Mon, NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
        );
    }

    #[test]
    fn test_get_parsed_time_4_minutes_to_0() {
        let time = DateTime::parse_from_rfc3339("2025-12-28T00:04:00Z")
            .unwrap()
            .to_utc();
        let res = get_parsed_time(time);
        assert_eq!(
            res,
            Some((Weekday::Sun, NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
        );
    }
}
