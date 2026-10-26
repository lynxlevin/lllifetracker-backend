use std::future::Future;

use chrono::{Duration, Utc};
use entities::{
    action,
    action_goal::{Column, Entity},
    user,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter,
};
use serde::Serialize;
use uuid::Uuid;

use crate::action_goal_service::{ActionGoalService, ActionGoalServiceError};
#[derive(Serialize, Debug, Clone)]
pub struct ActionGoalDiscontinueParams<'a> {
    pub user: &'a user::Model,
    pub action_id: Uuid,
}
pub trait ActionGoalServiceMutation {
    fn discontinue(
        &self,
        params: ActionGoalDiscontinueParams,
    ) -> impl Future<Output = Result<(), ActionGoalServiceError>>;
}

impl ActionGoalServiceMutation for ActionGoalService<'_> {
    async fn discontinue(&self, params: ActionGoalDiscontinueParams<'_>) -> Result<(), ActionGoalServiceError> {
        action::Entity::find_by_id(params.action_id)
            .filter(action::Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionGoalServiceError::ActionNotFound())?;

        let active_goal = Entity::find()
            .filter(Column::UserId.eq(params.user.id))
            .filter(Column::ActionId.eq(params.action_id))
            .filter(Column::ToDate.is_null())
            .one(self.db)
            .await?
            .ok_or(ActionGoalServiceError::ActionGoalNotFound())?;

        let user_today = params.user.timezone.convert_utc(Utc::now()).date_naive();
        if active_goal.from_date == user_today {
            active_goal.delete(self.db).await?;
        } else {
            let mut active_goal = active_goal.into_active_model();
            active_goal.to_date = Set(Some(user_today - Duration::days(1)));
            active_goal.update(self.db).await?;
        }

        Ok(())
    }
}
