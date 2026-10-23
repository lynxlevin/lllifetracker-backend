use std::future::Future;

use chrono::{DateTime, Duration, FixedOffset, Utc};
use entities::{
    action::{ActionTrackType, ActiveModel, Column, Entity, Model, DEFAULT_ACTION_COLOR},
    action_goal, action_track,
    tag::{self, TagType},
    user,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel,
    JoinType::LeftJoin, ModelTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, TransactionTrait,
};
use serde::Serialize;
use uuid::Uuid;

use crate::action_service::{ActionService, ActionServiceError};

#[derive(Serialize, Debug, Clone)]
pub struct ActionCreateParams {
    pub name: String,
    pub discipline: Option<String>,
    pub memo: Option<String>,
    pub color: Option<String>,
    pub track_type: ActionTrackType,
    pub user_id: Uuid,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionUpdateParams {
    pub name: String,
    pub discipline: Option<String>,
    pub memo: Option<String>,
    pub color: String,
    pub user_id: Uuid,
    pub action_id: Uuid,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionConvertTrackTypeParams {
    pub user: user::Model,
    pub action_id: Uuid,
    pub track_type: ActionTrackType,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionBulkUpdateOrderingParams {
    pub user_id: Uuid,
    pub action_ids_in_order: Vec<Uuid>,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionArchiveParams {
    pub user: user::Model,
    pub action_id: Uuid,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionUnarchiveParams {
    pub user: user::Model,
    pub action_id: Uuid,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionDeleteParams {
    pub user: user::Model,
    pub action_id: Uuid,
}
pub trait ActionServiceMutation {
    fn create(&self, params: ActionCreateParams) -> impl Future<Output = Result<Model, ActionServiceError>>;
    fn update(&self, params: ActionUpdateParams) -> impl Future<Output = Result<Model, ActionServiceError>>;
    fn convert_track_type(
        &self,
        params: ActionConvertTrackTypeParams,
    ) -> impl Future<Output = Result<Model, ActionServiceError>>;
    fn bulk_update_ordering(
        &self,
        params: ActionBulkUpdateOrderingParams,
    ) -> impl Future<Output = Result<(), ActionServiceError>>;
    fn archive(&self, params: ActionArchiveParams) -> impl Future<Output = Result<Model, ActionServiceError>>;
    fn unarchive(&self, params: ActionUnarchiveParams) -> impl Future<Output = Result<Model, ActionServiceError>>;
    fn delete(&self, params: ActionDeleteParams) -> impl Future<Output = Result<(), ActionServiceError>>;
}

impl ActionServiceMutation for ActionService<'_> {
    async fn create(&self, params: ActionCreateParams) -> Result<Model, ActionServiceError> {
        let action = self
            .db
            .transaction(|txn| {
                Box::pin(async move {
                    let action = ActiveModel {
                        id: Set(Uuid::now_v7()),
                        user_id: Set(params.user_id),
                        name: Set(params.name.to_owned()),
                        discipline: Set(params.discipline.to_owned()),
                        memo: Set(params.memo.to_owned()),
                        color: Set(params.color.unwrap_or(DEFAULT_ACTION_COLOR.to_string())),
                        track_type: Set(params.track_type),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    tag::ActiveModel {
                        id: Set(uuid::Uuid::now_v7()),
                        user_id: Set(params.user_id),
                        action_id: Set(Some(action.id)),
                        r#type: Set(TagType::Action),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    Ok(action)
                })
            })
            .await?;

        Ok(action)
    }

    async fn update(&self, params: ActionUpdateParams) -> Result<Model, ActionServiceError> {
        let action = Entity::find_by_id(params.action_id)
            .filter(Column::UserId.eq(params.user_id))
            .one(self.db)
            .await?
            .ok_or(ActionServiceError::ActionNotFound())?;

        let mut action = action.into_active_model();
        action.name = Set(params.name);
        action.discipline = Set(params.discipline);
        action.memo = Set(params.memo);
        action.color = Set(params.color);
        action.updated_at = Set(Utc::now().into());
        let action = action.update(self.db).await?;

        Ok(action)
    }

    async fn convert_track_type(&self, params: ActionConvertTrackTypeParams) -> Result<Model, ActionServiceError> {
        let action = Entity::find_by_id(params.action_id)
            .filter(Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionServiceError::ActionNotFound())?;

        if action.track_type == params.track_type {
            return Ok(action);
        }

        let action_goal = action_goal::Entity::find()
            .filter(action_goal::Column::ActionId.eq(action.id))
            .filter(action_goal::Column::UserId.eq(params.user.id))
            .filter(action_goal::Column::ToDate.is_null())
            .one(self.db)
            .await?;

        let action = self
            .db
            .transaction(|txn| {
                Box::pin(async move {
                    let mut action = action.into_active_model();
                    action.track_type = Set(params.track_type);
                    action.updated_at = Set(Utc::now().into());
                    let action = action.update(txn).await?;

                    if let Some(active_action_goal) = action_goal {
                        let user_today = params.user.timezone.convert_utc(Utc::now()).date_naive();
                        if active_action_goal.from_date == user_today {
                            active_action_goal.delete(txn).await?;
                        } else {
                            let mut active_action_goal = active_action_goal.into_active_model();
                            active_action_goal.to_date = Set(Some(user_today - Duration::days(1)));
                            active_action_goal.update(txn).await?;
                        }
                    }

                    Ok(action)
                })
            })
            .await?;

        Ok(action)
    }

    async fn bulk_update_ordering(
        &self,
        params: ActionBulkUpdateOrderingParams,
    ) -> Result<(), ActionServiceError> {
        let actions = Entity::find()
            .filter(Column::UserId.eq(params.user_id))
            .filter(Column::Id.is_in(params.action_ids_in_order.clone()))
            .all(self.db)
            .await?;

        self.db
            .transaction_async(async |txn| {
                for action in actions {
                    let order = params.action_ids_in_order.iter().position(|id| id == &action.id);
                    if let Some(order) = order {
                        let mut action = action.into_active_model();
                        action.ordering = Set(Some((order + 1) as i32));
                        action.update(txn).await?;
                    }
                }
                Ok(())
            })
            .await?;

        Ok(())
    }

    async fn archive(&self, params: ActionArchiveParams) -> Result<Model, ActionServiceError> {
        let action = Entity::find_by_id(params.action_id)
            .filter(Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionServiceError::ActionNotFound())?;

        let mut action = action.into_active_model();
        action.archived = Set(true);
        action.updated_at = Set(Utc::now().into());

        let action = self
            .db
            .transaction(|txn| {
                Box::pin(async move {
                    let action = action.update(txn).await?;

                    let first_track_at = resolve_first_track_at(txn, &params.user).await?;
                    update_user_first_track_at(txn, params.user, first_track_at).await?;

                    Ok(action)
                })
            })
            .await?;

        Ok(action)
    }

    async fn unarchive(&self, params: ActionUnarchiveParams) -> Result<Model, ActionServiceError> {
        let action = Entity::find_by_id(params.action_id)
            .filter(Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionServiceError::ActionNotFound())?;

        let mut action = action.into_active_model();
        action.archived = Set(false);
        action.updated_at = Set(Utc::now().into());

        let action = self
            .db
            .transaction(|txn| {
                Box::pin(async move {
                    let action = action.update(txn).await?;

                    let first_track_at = resolve_first_track_at(txn, &params.user).await?;
                    update_user_first_track_at(txn, params.user, first_track_at).await?;

                    Ok(action)
                })
            })
            .await?;

        Ok(action)
    }

    async fn delete(&self, params: ActionDeleteParams) -> Result<(), ActionServiceError> {
        let action = Entity::find_by_id(params.action_id)
            .filter(Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionServiceError::ActionNotFound())?;

        self.db
            .transaction(|txn| {
                Box::pin(async move {
                    action.clone().delete(txn).await?;

                    let first_track_at = resolve_first_track_at(txn, &params.user).await?;
                    update_user_first_track_at(txn, params.user, first_track_at).await?;

                    Ok(())
                })
            })
            .await?;

        Ok(())
    }
}

async fn resolve_first_track_at(
    db: &impl ConnectionTrait,
    user: &user::Model,
) -> Result<Option<DateTime<FixedOffset>>, ActionServiceError> {
    action_track::Entity::find()
        .select_only()
        .column(action_track::Column::StartedAt)
        .filter(action_track::Column::UserId.eq(user.id))
        .join(LeftJoin, action_track::Relation::Action.def())
        .filter(Column::Archived.eq(false))
        .order_by_asc(action_track::Column::StartedAt)
        .into_tuple()
        .one(db)
        .await
        .map_err(|e| e.into())
}

async fn update_user_first_track_at(
    db: &impl ConnectionTrait,
    user: user::Model,
    first_track_at: Option<DateTime<FixedOffset>>,
) -> Result<user::Model, ActionServiceError> {
    let mut user = user.into_active_model();
    user.first_track_at = Set(first_track_at);
    user.updated_at = Set(Utc::now().into());
    user.update(db).await.map_err(|e| e.into())
}
