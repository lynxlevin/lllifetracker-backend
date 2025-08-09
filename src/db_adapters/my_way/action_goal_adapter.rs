use std::future::Future;

use chrono::{NaiveDate, Utc};
use sea_orm::{
    sea_query::NullOrdering::Last, ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, Order, QueryFilter, QueryOrder, Select, Set, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use entities::action_goal::{ActiveModel, Column, Entity, Model};

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

// pub trait ActionGoalFilter {
//     fn filter_eq_user(self, user: &user::Model) -> Self;
//     fn filter_eq_archived(self, archived: bool) -> Self;
//     fn filter_in_ids(self, ids: Vec<Uuid>) -> Self;
// }

// impl ActionGoalFilter for ActionGoalAdapter<'_> {
//     fn filter_eq_user(mut self, user: &user::Model) -> Self {
//         self.query = self.query.filter(Column::UserId.eq(user.id));
//         self
//     }

//     fn filter_eq_archived(mut self, archived: bool) -> Self {
//         self.query = self.query.filter(Column::Archived.eq(archived));
//         self
//     }

//     fn filter_in_ids(mut self, ids: Vec<Uuid>) -> Self {
//         self.query = self.query.filter(Column::Id.is_in(ids));
//         self
//     }
// }

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

// pub trait ActionGoalQuery {
//     fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
//     fn get_all_with_goal(
//         self,
//     ) -> impl Future<Output = Result<Vec<(Model, Option<action_goal::Model>)>, DbErr>>;
//     fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
// }

// impl ActionGoalQuery for ActionGoalAdapter<'_> {
//     async fn get_all(self) -> Result<Vec<Model>, DbErr> {
//         self.query.all(self.db).await
//     }

//     async fn get_all_with_goal(self) -> Result<Vec<(Model, Option<action_goal::Model>)>, DbErr> {
//         self.query
//             .find_also_related(action_goal::Entity)
//             .all(self.db)
//             .await
//     }

//     async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
//         self.query.filter(Column::Id.eq(id)).one(self.db).await
//     }
// }

#[derive(Debug, Clone)]
pub struct CreateActionGoalParams {
    pub from_date: NaiveDate,
    pub duration_seconds: Option<i32>,
    pub count: Option<i32>,
    pub action_id: Uuid,
    pub user_id: Uuid,
}

// #[derive(Debug, Clone)]
// pub struct UpdateActionGoalParams {
//     pub name: String,
//     pub description: Option<String>,
//     pub trackable: Option<bool>,
//     pub color: Option<String>,
// }

pub trait ActionGoalMutation {
    fn create(self, params: CreateActionGoalParams) -> impl Future<Output = Result<Model, DbErr>>;
    //     fn update(
    //         self,
    //         action: Model,
    //         params: UpdateActionGoalParams,
    //     ) -> impl Future<Output = Result<Model, DbErr>>;
    //     fn delete(self, action: Model) -> impl Future<Output = Result<(), DbErr>>;
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

    //     async fn update(self, action: Model, params: UpdateActionGoalParams) -> Result<Model, DbErr> {
    //         let mut action = action.into_active_model();
    //         action.name = Set(params.name);
    //         action.description = Set(params.description);
    //         if let Some(trackable) = params.trackable {
    //             action.trackable = Set(trackable);
    //         }
    //         if let Some(color) = params.color {
    //             action.color = Set(color);
    //         }
    //         action.updated_at = Set(Utc::now().into());
    //         action.update(self.db).await
    //     }

    //     async fn delete(self, action: Model) -> Result<(), DbErr> {
    //         action.delete(self.db).await.map(|_| ())
    //     }
}
