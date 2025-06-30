use crate::{
    my_way::action_tracks::types::{
        ActionTrackAggregation, ActionTrackAggregationDuration, ActionTrackAggregationQuery,
    },
    UseCaseError,
};
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackOrder, ActionTrackQuery,
    },
    Order,
};
use entities::user as user_entity;

pub async fn aggregate_action_tracks<'a>(
    user: user_entity::Model,
    params: ActionTrackAggregationQuery,
    action_track_adapter: ActionTrackAdapter<'a>,
) -> Result<ActionTrackAggregation, UseCaseError> {
    let params = parse_params(params)?;
    let mut query = action_track_adapter.filter_eq_user(&user);
    if let Some(started_at_gte) = params.started_at_gte {
        query = query.filter_started_at_gte(started_at_gte)
    };
    if let Some(started_at_lte) = params.started_at_lte {
        query = query.filter_started_at_lte(started_at_lte)
    };
    if let Some(dates) = params.dates {
        query = query.filter_started_at_in_dates(dates, user.timezone)
    }
    let action_tracks = query
        .filter_ended_at_is_null(false)
        .filter_eq_archived_action(false)
        .order_by_action_id(Order::Asc)
        .order_by_started_at(Order::Desc)
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<ActionTrackAggregationDuration> = vec![];
    for action_track in action_tracks {
        if res.is_empty() || res.last().unwrap().action_id != action_track.action_id {
            res.push(ActionTrackAggregationDuration {
                action_id: action_track.action_id,
                duration: action_track.duration.unwrap_or(0),
                count: 1,
            });
        } else {
            res.last_mut().unwrap().duration += action_track.duration.unwrap_or(0);
            res.last_mut().unwrap().count += 1;
        }
    }
    Ok(ActionTrackAggregation {
        durations_by_action: res,
    })
}

#[derive(Default)]
struct ParsedParams {
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
    pub dates: Option<Vec<NaiveDate>>,
}

fn parse_params(params: ActionTrackAggregationQuery) -> Result<ParsedParams, UseCaseError> {
    match params.dates {
        Some(dates) => match params.started_at_gte.is_some() || params.started_at_lte.is_some() {
            true => Err(UseCaseError::BadRequest(
                "dates and started_at_gte/lte cannot be queried at the same time.".to_string(),
            )),
            false => {
                let parsed_dates: Vec<NaiveDate> = dates
                    .split(',')
                    .map(|date| {
                        let year = date[0..4].parse::<i32>().ok();
                        let month = date[4..6].parse::<u32>().ok();
                        let date = date[6..8].parse::<u32>().ok();
                        NaiveDate::from_ymd_opt(
                            year.unwrap_or(NaiveDate::MAX.year() + 1),
                            month.unwrap_or(0),
                            date.unwrap_or(0),
                        )
                    })
                    .filter(|date| date.is_some())
                    .map(|date| date.unwrap())
                    .collect();
                Ok(ParsedParams {
                    dates: Some(parsed_dates),
                    ..Default::default()
                })
            }
        },
        None => Ok(ParsedParams {
            started_at_gte: params.started_at_gte,
            started_at_lte: params.started_at_lte,
            ..Default::default()
        }),
    }
}
