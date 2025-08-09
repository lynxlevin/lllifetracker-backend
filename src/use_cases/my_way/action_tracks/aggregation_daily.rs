use std::collections::HashMap;

use crate::{
    my_way::action_tracks::types::{
        ActionTrackAggregationDailyQuery, ActionTrackAggregationDuration,
        ActionTrackDailyAggregationItem,
    },
    UseCaseError,
};
use chrono::{DateTime, Datelike, Duration, FixedOffset};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackOrder, ActionTrackQuery,
    },
    Order,
};
use entities::{
    custom_methods::user::UserTimezoneTrait, sea_orm_active_enums::TimezoneEnum,
    user as user_entity,
};

pub async fn aggregate_daily_action_tracks<'a>(
    user: user_entity::Model,
    params: ActionTrackAggregationDailyQuery,
    action_track_adapter: ActionTrackAdapter<'a>,
) -> Result<HashMap<String, Vec<ActionTrackDailyAggregationItem>>, UseCaseError> {
    let params = parse_params(params, &user.timezone)?;
    let action_tracks = action_track_adapter
        .filter_eq_user(&user)
        .filter_started_at_gte(params.start)
        .filter_started_at_lte(params.end)
        .filter_ended_at_is_null(false)
        .filter_eq_archived_action(false)
        .order_by_started_at(Order::Desc)
        .order_by_action_id(Order::Asc)
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut aggregation_items: Vec<ActionTrackDailyAggregationItem> = vec![];
    for track in action_tracks {
        let date = user.to_user_timezone(track.started_at.to_utc()).day();
        if aggregation_items.is_empty() || aggregation_items.last().unwrap().date != date {
            aggregation_items.push(ActionTrackDailyAggregationItem {
                date,
                aggregation: vec![ActionTrackAggregationDuration {
                    action_id: track.action_id,
                    duration: track.duration.unwrap(),
                    count: 1,
                }],
            })
        } else {
            let item = aggregation_items.last_mut().unwrap();
            match item
                .aggregation
                .iter_mut()
                .find(|agg| agg.action_id == track.action_id)
            {
                Some(agg_item) => {
                    agg_item.duration += track.duration.unwrap();
                    agg_item.count += 1;
                }
                None => item.aggregation.push(ActionTrackAggregationDuration {
                    action_id: track.action_id,
                    duration: track.duration.unwrap(),
                    count: 1,
                }),
            }
        }
    }

    let mut res: HashMap<String, Vec<ActionTrackDailyAggregationItem>> = HashMap::new();
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
) -> Result<ParsedParams, UseCaseError> {
    let year: u32 = params.year_month[0..4]
        .parse()
        .map_err(|_| UseCaseError::BadRequest(INVALID_YEAR_MONTH_MESSAGE.to_string()))?;
    let month: u32 = params.year_month[4..6]
        .parse()
        .map_err(|_| UseCaseError::BadRequest(INVALID_YEAR_MONTH_MESSAGE.to_string()))?;

    let utc_start = DateTime::parse_from_rfc3339(&format!("{year}-{:0>2}-01T00:00:00Z", month))
        .map_err(|_| UseCaseError::BadRequest(INVALID_YEAR_MONTH_MESSAGE.to_string()))?;

    let utc_end = match month == 12 {
        true => DateTime::parse_from_rfc3339(&format!("{}-01-01T23:59:59Z", year + 1)),
        false => DateTime::parse_from_rfc3339(&format!("{year}-{:0>2}-01T23:59:59Z", month + 1)),
    }
    .map_err(|_| UseCaseError::BadRequest(INVALID_YEAR_MONTH_MESSAGE.to_string()))?
        - Duration::days(1);

    match user_timezone {
        TimezoneEnum::AsiaTokyo => Ok(ParsedParams {
            start: utc_start - Duration::hours(9),
            end: utc_end - Duration::hours(9),
            year_month: params.year_month,
        }),
        TimezoneEnum::Utc => Ok(ParsedParams {
            start: utc_start,
            end: utc_end,
            year_month: params.year_month,
        }),
    }
}
