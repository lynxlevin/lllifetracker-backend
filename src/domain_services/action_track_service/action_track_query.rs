use std::future::Future;

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
use entities::{
    action,
    action_track::{Column, Entity, Model, Relation},
    user::TimezoneEnum,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::action_track_service::{ActionTrackService, ActionTrackServiceError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionTrackListParams {
    pub active_only: Option<bool>,
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionTrackAggregateParams {
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
    pub dates: Option<Vec<NaiveDate>>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ActionTrackAggregateResponse {
    pub durations_by_action: Vec<ActionTrackAggregateDuration>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ActionTrackAggregateDuration {
    pub action_id: uuid::Uuid,
    pub duration: i64,
    pub count: i64,
}
pub trait ActionTrackServiceQuery {
    fn list(
        &self,
        user_id: Uuid,
        params: ActionTrackListParams,
    ) -> impl Future<Output = Result<Vec<Model>, ActionTrackServiceError>>;
    fn aggregate(
        &self,
        user_id: Uuid,
        user_timezone: TimezoneEnum,
        params: ActionTrackAggregateParams,
    ) -> impl Future<Output = Result<ActionTrackAggregateResponse, ActionTrackServiceError>>;
}

impl ActionTrackServiceQuery for ActionTrackService<'_> {
    async fn list(
        &self,
        user_id: Uuid,
        params: ActionTrackListParams,
    ) -> Result<Vec<Model>, ActionTrackServiceError> {
        let mut query = Entity::find()
            .join(LeftJoin, Relation::Action.def())
            .filter(Column::UserId.eq(user_id))
            .filter(action::Column::Archived.eq(false));
        if params.active_only.is_some_and(|active_only| active_only) {
            query = query.filter(Column::EndedAt.is_null());
        }
        if let Some(started_at_gte) = params.started_at_gte {
            query = query.filter(Column::StartedAt.gte(started_at_gte));
        }
        if let Some(started_at_lte) = params.started_at_lte {
            query = query.filter(Column::StartedAt.lte(started_at_lte));
        }
        let action_tracks = query.order_by_desc(Column::StartedAt).all(self.db).await?;

        Ok(action_tracks)
    }
    async fn aggregate(
        &self,
        user_id: Uuid,
        user_timezone: TimezoneEnum,
        params: ActionTrackAggregateParams,
    ) -> Result<ActionTrackAggregateResponse, ActionTrackServiceError> {
        let mut query = Entity::find()
            .join(LeftJoin, Relation::Action.def())
            .filter(Column::UserId.eq(user_id))
            .filter(action::Column::Archived.eq(false))
            .filter(Column::EndedAt.is_not_null());
        if let Some(started_at_gte) = params.started_at_gte {
            query = query.filter(Column::StartedAt.gte(started_at_gte));
        }
        if let Some(started_at_lte) = params.started_at_lte {
            query = query.filter(Column::StartedAt.lte(started_at_lte));
        }
        if let Some(dates) = params.dates {
            let mut cond = Condition::any();
            for date in dates {
                let (start, end) = get_date_start_end_in_utc(date, &user_timezone);
                cond = cond.add(Column::StartedAt.between(start, end))
            }
            query = query.filter(cond);
        }
        let action_tracks = query
            .order_by_asc(Column::ActionId)
            .order_by_desc(Column::StartedAt)
            .all(self.db)
            .await?;

        let duration: Vec<ActionTrackAggregateDuration> =
            action_tracks.into_iter().fold(vec![], |mut duration, track| {
                if duration.is_empty() || duration.last().unwrap().action_id != track.action_id {
                    duration.push(ActionTrackAggregateDuration {
                        action_id: track.action_id,
                        duration: track.duration.unwrap_or(0),
                        count: 1,
                    });
                } else {
                    duration.last_mut().unwrap().duration += track.duration.unwrap_or(0);
                    duration.last_mut().unwrap().count += 1;
                }
                duration
            });
        Ok(ActionTrackAggregateResponse { durations_by_action: duration })
    }
}

fn get_date_start_end_in_utc(date: NaiveDate, user_timezone: &TimezoneEnum) -> (DateTime<Utc>, DateTime<Utc>) {
    let utc_start = date.and_hms_micro_opt(0, 0, 0, 0).unwrap().and_utc();
    let utc_end = date.and_hms_micro_opt(23, 59, 59, 999999).unwrap().and_utc();
    match user_timezone {
        TimezoneEnum::AsiaTokyo => (utc_start - Duration::hours(9), utc_end - Duration::hours(9)),
        TimezoneEnum::Utc => (utc_start, utc_end),
    }
}
