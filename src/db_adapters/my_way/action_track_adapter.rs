use std::future::Future;

use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::{
    sqlx::error::Error::Database, ActiveModelTrait, ColumnTrait, Condition, DbConn, DbErr,
    EntityTrait, IntoActiveModel, JoinType::LeftJoin, ModelTrait, Order, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, RuntimeErr::SqlxError, Select, Set,
};
use uuid::Uuid;

use crate::CustomDbErr;
use entities::{
    action,
    action_track::{ActiveModel, Column, Entity, Model, Relation},
    user,
};

#[derive(Clone)]
pub struct ActionTrackAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> ActionTrackAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait ActionTrackFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_started_at_gte(self, started_at: DateTime<FixedOffset>) -> Self;
    fn filter_started_at_lte(self, started_at: DateTime<FixedOffset>) -> Self;
    fn filter_started_at_in_dates(self, dates: Vec<NaiveDate>) -> Self;
    fn filter_ended_at_is_null(self, is_null: bool) -> Self;
    fn filter_eq_archived_action(self, archived: bool) -> Self;
}

impl ActionTrackFilter for ActionTrackAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }

    fn filter_started_at_gte(mut self, started_at: DateTime<FixedOffset>) -> Self {
        self.query = self.query.filter(Column::StartedAt.gte(started_at));
        self
    }

    fn filter_started_at_lte(mut self, started_at: DateTime<FixedOffset>) -> Self {
        self.query = self.query.filter(Column::StartedAt.lte(started_at));
        self
    }

    fn filter_started_at_in_dates(mut self, dates: Vec<NaiveDate>) -> Self {
        let mut cond = Condition::any();
        for date in dates {
            cond = cond.add(
                Column::StartedAt.between(
                    // FIXME: Need to take the user's timezone into account.
                    date.pred_opt()
                        .unwrap()
                        .and_hms_micro_opt(15, 0, 0, 0)
                        .unwrap(),
                    date.and_hms_micro_opt(14, 59, 59, 999999).unwrap(),
                ),
            )
        }
        self.query = self.query.filter(cond);
        self
    }

    fn filter_ended_at_is_null(mut self, is_null: bool) -> Self {
        self.query = match is_null {
            true => self.query.filter(Column::EndedAt.is_null()),
            false => self.query.filter(Column::EndedAt.is_not_null()),
        };
        self
    }

    fn filter_eq_archived_action(mut self, archived: bool) -> Self {
        self.query = self
            .query
            .join(LeftJoin, Relation::Action.def())
            .filter(action::Column::Archived.eq(archived));
        self
    }
}

pub trait ActionTrackOrder {
    fn order_by_action_id(self, order: Order) -> Self;
    fn order_by_started_at(self, order: Order) -> Self;
}

impl ActionTrackOrder for ActionTrackAdapter<'_> {
    fn order_by_action_id(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::ActionId, order);
        self
    }

    fn order_by_started_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::StartedAt, order);
        self
    }
}

pub trait ActionTrackQuery {
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl ActionTrackQuery for ActionTrackAdapter<'_> {
    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateActionTrackParams {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration: Option<i64>,
    pub action_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateActionTrackParams {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration: Option<i64>,
    pub action_id: uuid::Uuid,
}

pub trait ActionTrackMutation {
    fn create(self, params: CreateActionTrackParams) -> impl Future<Output = Result<Model, DbErr>>;
    fn update(
        self,
        action_track: Model,
        params: UpdateActionTrackParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn delete(self, action_track: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl ActionTrackMutation for ActionTrackAdapter<'_> {
    async fn create(self, params: CreateActionTrackParams) -> Result<Model, DbErr> {
        ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(params.user_id),
            action_id: Set(params.action_id),
            started_at: Set(params.started_at),
            ended_at: Set(params.ended_at),
            duration: Set(params.duration),
        }
        .insert(self.db)
        .await
        .map_err(|e| match &e {
            DbErr::Query(SqlxError(Database(error))) => match error.constraint() {
                Some("action_tracks_user_id_action_id_started_at_unique_index") => {
                    DbErr::Custom(CustomDbErr::Duplicate.to_string())
                }
                _ => e,
            },
            _ => e,
        })
    }

    async fn update(
        self,
        action_track: Model,
        params: UpdateActionTrackParams,
    ) -> Result<Model, DbErr> {
        let mut action_track = action_track.into_active_model();
        action_track.started_at = Set(params.started_at);
        action_track.ended_at = Set(params.ended_at);
        action_track.duration = Set(params.duration);
        action_track.action_id = Set(params.action_id);
        action_track.update(self.db).await.map_err(|e| match &e {
            DbErr::Query(SqlxError(Database(error))) => match error.constraint() {
                Some("action_tracks_user_id_action_id_started_at_unique_index") => {
                    DbErr::Custom(CustomDbErr::Duplicate.to_string())
                }
                _ => e,
            },
            _ => e,
        })
    }

    async fn delete(self, action_track: Model) -> Result<(), DbErr> {
        action_track.delete(self.db).await.map(|_| ())
    }
}
