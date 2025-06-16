use chrono::{DateTime, FixedOffset};
use sea_orm::{
    ColumnTrait, DbConn, DbErr, EntityTrait, JoinType::LeftJoin, Order, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Select,
};
use uuid::Uuid;

use entities::{
    action,
    action_track::{Column, Entity, Model, Relation},
    user,
};

#[derive(Clone)]
pub struct ActionTrackQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> ActionTrackQuery<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }

    pub async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }

    pub async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }
}

pub trait ActionTrackQueryFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_started_at_gte(self, started_at: DateTime<FixedOffset>) -> Self;
    fn filter_started_at_lte(self, started_at: DateTime<FixedOffset>) -> Self;
    fn filter_ended_at_is_null(self, is_null: bool) -> Self;
    fn filter_eq_archived_action(self, archived: bool) -> Self;
}

impl ActionTrackQueryFilter for ActionTrackQuery<'_> {
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

pub trait ActionTrackQueryOrder {
    fn order_by_action_id(self, order: Order) -> Self;
    fn order_by_started_at(self, order: Order) -> Self;
}

impl ActionTrackQueryOrder for ActionTrackQuery<'_> {
    fn order_by_action_id(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::ActionId, order);
        self
    }

    fn order_by_started_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::StartedAt, order);
        self
    }
}
