use std::future::Future;

use chrono::{Duration, NaiveDate, Utc};
use entities::{
    action::{self, ActionTrackType},
    action_goal::{ActiveModel, Column, Entity, Model},
    user,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
    ModelTrait, QueryFilter,
};
use serde::Serialize;
use uuid::Uuid;

use crate::action_goal_service::{ActionGoalService, ActionGoalServiceError};
#[derive(Serialize, Debug, Clone)]
pub struct ActionGoalSetNewParams<'a> {
    pub user: &'a user::Model,
    pub action_id: Uuid,
    pub duration_seconds: Option<i32>,
    pub count: Option<i32>,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionGoalDiscontinueParams<'a> {
    pub user: &'a user::Model,
    pub action_id: Uuid,
}
pub trait ActionGoalServiceMutation {
    fn set_new(
        &self,
        params: ActionGoalSetNewParams,
    ) -> impl Future<Output = Result<Model, ActionGoalServiceError>>;
    fn discontinue(
        &self,
        params: ActionGoalDiscontinueParams,
    ) -> impl Future<Output = Result<(), ActionGoalServiceError>>;
}

impl ActionGoalServiceMutation for ActionGoalService<'_> {
    async fn set_new(&self, params: ActionGoalSetNewParams<'_>) -> Result<Model, ActionGoalServiceError> {
        let action = action::Entity::find_by_id(params.action_id)
            .filter(action::Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionGoalServiceError::ActionNotFound())?;

        match action.track_type {
            ActionTrackType::TimeSpan => {
                if params.duration_seconds.is_none() {
                    return Err(ActionGoalServiceError::DurationSecondsNullForTimeSpanAction());
                }
                if params.count.is_some() {
                    return Err(ActionGoalServiceError::CountNotNullForTimeSpanAction());
                }
            }
            ActionTrackType::Count => {
                if params.count.is_none() {
                    return Err(ActionGoalServiceError::CountNullForCountAction());
                }
                if params.duration_seconds.is_some() {
                    return Err(ActionGoalServiceError::DurationSecondsNotNullForCountAction());
                }
            }
        };

        let user_today = get_user_today(params.user);
        let active_goal = Entity::find()
            .filter(Column::UserId.eq(params.user.id))
            .filter(Column::ActionId.eq(params.action_id))
            .filter(Column::ToDate.is_null())
            .one(self.db)
            .await?;
        let new_goal = self
            .db
            .transaction_async(async |txn| {
                let goal = match active_goal {
                    Some(active_goal) => {
                        if active_goal.from_date == user_today {
                            let mut active_goal = active_goal.into_active_model();
                            active_goal.duration_seconds = Set(params.duration_seconds);
                            active_goal.count = Set(params.count);
                            active_goal.update(txn).await?
                        } else {
                            let mut active_goal = active_goal.into_active_model();
                            active_goal.to_date = Set(Some(user_today - Duration::days(1)));
                            active_goal.update(txn).await?;
                            create_new_goal_starting_today(txn, params).await?
                        }
                    }
                    None => create_new_goal_starting_today(txn, params).await?,
                };
                Ok(goal)
            })
            .await?;

        Ok(new_goal)
    }

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

        let user_today = get_user_today(params.user);
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

fn get_user_today(user: &user::Model) -> NaiveDate {
    user.timezone.convert_utc(Utc::now()).date_naive()
}

async fn create_new_goal_starting_today(
    db: &impl ConnectionTrait,
    params: ActionGoalSetNewParams<'_>,
) -> Result<Model, DbErr> {
    ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(params.user.id),
        action_id: Set(params.action_id),
        from_date: Set(get_user_today(params.user)),
        duration_seconds: Set(params.duration_seconds),
        count: Set(params.count),
        ..Default::default()
    }
    .insert(db)
    .await
}
