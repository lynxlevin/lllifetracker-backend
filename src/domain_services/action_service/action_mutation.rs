use std::future::Future;

use chrono::{DateTime, FixedOffset, Utc};
use entities::{
    action::{Column, Entity},
    action_track, user,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel,
    JoinType::LeftJoin, ModelTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, TransactionTrait,
};
use serde::Serialize;
use uuid::Uuid;

use crate::action_service::{ActionService, ActionServiceError};

#[derive(Serialize, Debug, Clone)]
pub struct ActionDeleteParams {
    pub user: user::Model,
    pub action_id: Uuid,
}
pub trait ActionServiceMutation {
    fn delete(&self, params: ActionDeleteParams) -> impl Future<Output = Result<(), ActionServiceError>>;
}

impl ActionServiceMutation for ActionService<'_> {
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
