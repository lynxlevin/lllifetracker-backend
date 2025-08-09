use std::future::Future;

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use sea_orm::{
    sea_query::NullOrdering::Last, ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, Order, QueryFilter, QueryOrder, Select, Set, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use entities::{
    action,
    action_goal::{ActiveModel, Column, Entity, Model},
    user,
};

#[derive(Clone)]
pub struct ActionGoalAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> ActionGoalAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait ActionGoalFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_eq_action(self, action: &action::Model) -> Self;
    fn filter_to_date_null(self) -> Self;
}

impl ActionGoalFilter for ActionGoalAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }

    fn filter_eq_action(mut self, action: &action::Model) -> Self {
        self.query = self.query.filter(Column::ActionId.eq(action.id));
        self
    }

    fn filter_to_date_null(mut self) -> Self {
        self.query = self.query.filter(Column::ToDate.is_null());
        self
    }
}

// pub trait ActionGoalOrder {
//     fn order_by_ordering_nulls_last(self, order: Order) -> Self;
//     fn order_by_created_at(self, order: Order) -> Self;
// }

// impl ActionGoalOrder for ActionGoalAdapter<'_> {
//     fn order_by_ordering_nulls_last(mut self, order: Order) -> Self {
//         self.query = self
//             .query
//             .order_by_with_nulls(Column::Ordering, order, Last);
//         self
//     }

//     fn order_by_created_at(mut self, order: Order) -> Self {
//         self.query = self.query.order_by(Column::CreatedAt, order);
//         self
//     }
// }

pub trait ActionGoalQuery {
    // fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
    // fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
    fn get_one(self) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl ActionGoalQuery for ActionGoalAdapter<'_> {
    // async fn get_all(self) -> Result<Vec<Model>, DbErr> {
    //     self.query.all(self.db).await
    // }

    // async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
    //     self.query.filter(Column::Id.eq(id)).one(self.db).await
    // }

    async fn get_one(self) -> Result<Option<Model>, DbErr> {
        self.query.one(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateActionGoalParams {
    pub from_date: NaiveDate,
    pub duration_seconds: Option<i32>,
    pub count: Option<i32>,
    pub action_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateActionGoalParams {
    pub to_date: Option<NaiveDate>,
    pub duration_seconds: Option<i32>,
    pub count: Option<i32>,
}

pub trait ActionGoalMutation {
    fn create(self, params: CreateActionGoalParams) -> impl Future<Output = Result<Model, DbErr>>;
    fn update(
        self,
        params: UpdateActionGoalParams,
        action_goal: Model,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn delete(self, action_goal: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl ActionGoalMutation for ActionGoalAdapter<'_> {
    async fn create(self, params: CreateActionGoalParams) -> Result<Model, DbErr> {
        let id = uuid::Uuid::now_v7();
        ActiveModel {
            id: Set(id),
            user_id: Set(params.user_id),
            action_id: Set(params.action_id),
            from_date: Set(params.from_date),
            duration_seconds: Set(params.duration_seconds),
            count: Set(params.count),
            ..Default::default()
        }
        .insert(self.db)
        .await
    }

    async fn update(
        self,
        params: UpdateActionGoalParams,
        action_goal: Model,
    ) -> Result<Model, DbErr> {
        let mut action_goal = action_goal.into_active_model();
        action_goal.to_date = Set(params.to_date);
        action_goal.duration_seconds = Set(params.duration_seconds);
        action_goal.count = Set(params.count);
        action_goal.update(self.db).await
    }

    async fn delete(self, action_goal: Model) -> Result<(), DbErr> {
        action_goal.delete(self.db).await.map(|_| ())
    }
}
