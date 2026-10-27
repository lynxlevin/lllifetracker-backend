use std::future::Future;

use entities::{
    action::{Column, Entity, Model, Relation},
    action_goal,
};
use sea_orm::{
    sea_query::NullOrdering, ColumnTrait, EntityTrait, JoinType::LeftJoin, Order, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::action_service::{ActionService, ActionServiceError};

pub trait ActionServiceQuery {
    fn list(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<(Model, Option<action_goal::Model>)>, ActionServiceError>>;
}

impl ActionServiceQuery for ActionService<'_> {
    async fn list(&self, user_id: Uuid) -> Result<Vec<(Model, Option<action_goal::Model>)>, ActionServiceError> {
        let actions = Entity::find()
            .join(
                LeftJoin,
                Relation::ActionGoal
                    .def()
                    .on_condition(|_left, _right| action_goal::Column::ToDate.is_null().into()),
            )
            .filter(Column::UserId.eq(user_id))
            .order_by_with_nulls(Column::Ordering, Order::Asc, NullOrdering::Last)
            .order_by_asc(Column::CreatedAt)
            .select_also(action_goal::Entity)
            .all(self.db)
            .await?;

        Ok(actions)
    }
}
