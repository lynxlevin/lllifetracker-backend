use std::future::Future;

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Utc};
use entities::{
    action,
    action::{Column, Entity, Model, Relation},
    user::TimezoneEnum,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::action_service::{ActionService, ActionServiceError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionListParams {
    pub active_only: Option<bool>,
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionAggregateParams {
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
    pub dates: Option<Vec<NaiveDate>>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ActionAggregateResponse {
    pub durations_by_action: Vec<ActionAggregateDuration>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ActionAggregateDuration {
    pub action_id: uuid::Uuid,
    pub duration: i64,
    pub count: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionAggregateByDateParams {
    pub started_at_gte: DateTime<FixedOffset>,
    pub started_at_lte: DateTime<FixedOffset>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ActionAggregateByDateResponseItem {
    pub date: u32,
    pub aggregation: Vec<ActionAggregateDuration>,
}
pub trait ActionServiceQuery {
    fn list(
        &self,
        user_id: Uuid,
        params: ActionListParams,
    ) -> impl Future<Output = Result<Vec<Model>, ActionServiceError>>;
    fn aggregate(
        &self,
        user_id: Uuid,
        user_timezone: TimezoneEnum,
        params: ActionAggregateParams,
    ) -> impl Future<Output = Result<ActionAggregateResponse, ActionServiceError>>;
    fn aggregate_by_date(
        &self,
        user_id: Uuid,
        user_timezone: TimezoneEnum,
        params: ActionAggregateByDateParams,
    ) -> impl Future<Output = Result<Vec<ActionAggregateByDateResponseItem>, ActionServiceError>>;
}

impl ActionServiceQuery for ActionService<'_> {
    async fn list(&self, user_id: Uuid, params: ActionListParams) -> Result<Vec<Model>, ActionServiceError> {
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
        let actions = query.order_by_desc(Column::StartedAt).all(self.db).await?;

        Ok(actions)
    }

    async fn aggregate(
        &self,
        user_id: Uuid,
        user_timezone: TimezoneEnum,
        params: ActionAggregateParams,
    ) -> Result<ActionAggregateResponse, ActionServiceError> {
        let mut query = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .join(LeftJoin, Relation::Action.def())
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
        let actions = query
            .order_by_asc(Column::ActionId)
            .order_by_desc(Column::StartedAt)
            .all(self.db)
            .await?;

        let duration: Vec<ActionAggregateDuration> = actions.into_iter().fold(vec![], |mut duration, track| {
            if duration.is_empty() || duration.last().unwrap().action_id != track.action_id {
                duration.push(ActionAggregateDuration {
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
        Ok(ActionAggregateResponse { durations_by_action: duration })
    }

    async fn aggregate_by_date(
        &self,
        user_id: Uuid,
        user_timezone: TimezoneEnum,
        params: ActionAggregateByDateParams,
    ) -> Result<Vec<ActionAggregateByDateResponseItem>, ActionServiceError> {
        let actions = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .join(LeftJoin, Relation::Action.def())
            .filter(action::Column::Archived.eq(false))
            .filter(Column::EndedAt.is_not_null())
            .filter(Column::StartedAt.gte(params.started_at_gte))
            .filter(Column::StartedAt.lte(params.started_at_lte))
            .order_by_desc(Column::StartedAt)
            .order_by_asc(Column::ActionId)
            .all(self.db)
            .await?;

        let res: Vec<ActionAggregateByDateResponseItem> = actions.into_iter().fold(vec![], |mut acc, track| {
            let date = user_timezone.convert_utc(track.started_at.to_utc()).day();
            if acc.is_empty() || acc.last().unwrap().date != date {
                acc.push(ActionAggregateByDateResponseItem {
                    date,
                    aggregation: vec![ActionAggregateDuration {
                        action_id: track.action_id,
                        duration: track.duration.unwrap(),
                        count: 1,
                    }],
                })
            } else {
                let item = acc.last_mut().unwrap();
                match item.aggregation.iter_mut().find(|agg| agg.action_id == track.action_id) {
                    Some(agg_item) => {
                        agg_item.duration += track.duration.unwrap();
                        agg_item.count += 1;
                    }
                    None => item.aggregation.push(ActionAggregateDuration {
                        action_id: track.action_id,
                        duration: track.duration.unwrap(),
                        count: 1,
                    }),
                }
            }
            acc
        });

        Ok(res)
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
