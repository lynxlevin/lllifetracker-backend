use std::collections::HashMap;

use crate::my_way::action_tracks::types::{
    ActionTrackAggregateByDateResponseItem, ActionTrackAggregationDailyQuery,
};
use chrono::{DateTime, Duration, FixedOffset};
use domain_services::action_track_service::{
    ActionTrackAggregateByDateParams, ActionTrackService, ActionTrackServiceError, ActionTrackServiceQuery,
};
use entities::user::{self as user_entity, TimezoneEnum};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AggregateDailyActionTracksError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionTrackServiceError> for AggregateDailyActionTracksError {
    fn from(e: ActionTrackServiceError) -> Self {
        AggregateDailyActionTracksError::InternalServerError(e.to_string())
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        params.year_month = params.year_month,
    ),
    skip_all
)]
pub async fn aggregate_daily_action_tracks<'a>(
    user: user_entity::Model,
    params: ActionTrackAggregationDailyQuery,
    action_track_service: ActionTrackService<'a>,
) -> Result<HashMap<String, Vec<ActionTrackAggregateByDateResponseItem>>, AggregateDailyActionTracksError> {
    let params = parse_params(params, &user.timezone)?;
    let aggregation_items = action_track_service
        .aggregate_by_date(
            user.id,
            user.timezone,
            ActionTrackAggregateByDateParams { started_at_gte: params.start, started_at_lte: params.end },
        )
        .await?;

    let mut res: HashMap<String, Vec<ActionTrackAggregateByDateResponseItem>> = HashMap::new();
    res.insert(params.year_month, aggregation_items);
    Ok(res)
}

#[derive(Default)]
struct ParsedParams {
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
    pub year_month: String,
}

const INVALID_YEAR_MONTH_MESSAGE: &str = "Year_month cannot be parsed into a date.";

fn parse_params(
    params: ActionTrackAggregationDailyQuery,
    user_timezone: &TimezoneEnum,
) -> Result<ParsedParams, AggregateDailyActionTracksError> {
    let year: u32 = params.year_month[0..4]
        .parse()
        .map_err(|_| AggregateDailyActionTracksError::ValidationError(INVALID_YEAR_MONTH_MESSAGE.to_string()))?;
    let month: u32 = params.year_month[4..6]
        .parse()
        .map_err(|_| AggregateDailyActionTracksError::ValidationError(INVALID_YEAR_MONTH_MESSAGE.to_string()))?;

    let utc_start = DateTime::parse_from_rfc3339(&format!("{year}-{:0>2}-01T00:00:00Z", month))
        .map_err(|_| AggregateDailyActionTracksError::ValidationError(INVALID_YEAR_MONTH_MESSAGE.to_string()))?;

    let utc_end = match month == 12 {
        true => DateTime::parse_from_rfc3339(&format!("{}-01-01T23:59:59Z", year + 1)),
        false => DateTime::parse_from_rfc3339(&format!("{year}-{:0>2}-01T23:59:59Z", month + 1)),
    }
    .map_err(|_| AggregateDailyActionTracksError::ValidationError(INVALID_YEAR_MONTH_MESSAGE.to_string()))?
        - Duration::days(1);

    match user_timezone {
        TimezoneEnum::AsiaTokyo => Ok(ParsedParams {
            start: utc_start - Duration::hours(9),
            end: utc_end - Duration::hours(9),
            year_month: params.year_month,
        }),
        TimezoneEnum::Utc => Ok(ParsedParams { start: utc_start, end: utc_end, year_month: params.year_month }),
    }
}
