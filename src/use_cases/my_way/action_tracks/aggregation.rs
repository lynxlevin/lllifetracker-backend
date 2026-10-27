use crate::my_way::action_tracks::types::{
    ActionTrackAggregateDuration, ActionTrackAggregateResponse, ActionTrackAggregationQuery,
};
use chrono::{Datelike, NaiveDate};
use domain_services::action_track_service::{
    ActionTrackAggregateParams, ActionTrackService, ActionTrackServiceError, ActionTrackServiceQuery,
};
use entities::user as user_entity;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AggregateActionTracksError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionTrackServiceError> for AggregateActionTracksError {
    fn from(e: ActionTrackServiceError) -> Self {
        Self::InternalServerError(e.to_string())
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        params.started_at_gte.is_some = params.started_at_gte.is_some(),
        params.started_at_lte.is_some = params.started_at_lte.is_some(),
        params.dates.is_some = params.dates.is_some(),
    ),
    skip_all
)]
pub async fn aggregate_action_tracks<'a>(
    user: user_entity::Model,
    params: ActionTrackAggregationQuery,
    action_track_service: ActionTrackService<'a>,
) -> Result<ActionTrackAggregateResponse, AggregateActionTracksError> {
    let params = parse_params(params)?;
    let res = action_track_service.aggregate(user.id, user.timezone, params).await?;
    Ok(ActionTrackAggregateResponse {
        durations_by_action: res
            .durations_by_action
            .iter()
            .map(|d| ActionTrackAggregateDuration { action_id: d.action_id, duration: d.duration, count: d.count })
            .collect(),
    })
}

fn parse_params(
    params: ActionTrackAggregationQuery,
) -> Result<ActionTrackAggregateParams, AggregateActionTracksError> {
    match params.dates {
        Some(dates) => match params.started_at_gte.is_some() || params.started_at_lte.is_some() {
            true => Err(AggregateActionTracksError::ValidationError(
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
                Ok(ActionTrackAggregateParams {
                    started_at_gte: None,
                    started_at_lte: None,
                    dates: Some(parsed_dates),
                })
            }
        },
        None => Ok(ActionTrackAggregateParams {
            started_at_gte: params.started_at_gte,
            started_at_lte: params.started_at_lte,
            dates: None,
        }),
    }
}
