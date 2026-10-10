use std::future::Future;

use chrono::{DateTime, FixedOffset, SubsecRound, Utc};
use entities::{
    action::{self, ActionTrackType},
    action_track::{ActiveModel, Column, Entity, Model, Relation},
    user,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel,
    JoinType::LeftJoin, ModelTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, TransactionTrait,
};
use serde::Serialize;
use uuid::Uuid;

use crate::action_track_service::{handle_unique_constraint, ActionTrackService, ActionTrackServiceError};

#[derive(Serialize, Debug, Clone)]
pub struct ActionTrackCreateParams {
    pub user: user::Model,
    pub action_id: Uuid,
    pub started_at: DateTime<FixedOffset>,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionTrackUpdateParams {
    pub user: user::Model,
    pub action_track_id: Uuid,
    pub action_id: Uuid,
    pub started_at: DateTime<FixedOffset>,
    pub ended_at: Option<DateTime<FixedOffset>>,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionTrackDeleteParams {
    pub user: user::Model,
    pub action_track_id: Uuid,
}
pub trait ActionTrackServiceMutation {
    fn create(
        &self,
        params: ActionTrackCreateParams,
    ) -> impl Future<Output = Result<Model, ActionTrackServiceError>>;
    fn update(
        &self,
        params: ActionTrackUpdateParams,
    ) -> impl Future<Output = Result<Model, ActionTrackServiceError>>;
    fn delete(&self, params: ActionTrackDeleteParams)
        -> impl Future<Output = Result<(), ActionTrackServiceError>>;
}

impl ActionTrackServiceMutation for ActionTrackService<'_> {
    async fn create(&self, params: ActionTrackCreateParams) -> Result<Model, ActionTrackServiceError> {
        let action = action::Entity::find_by_id(params.action_id)
            .filter(action::Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionTrackServiceError::ActionNotFound())?;

        let started_at = params.started_at.trunc_subsecs(0);
        let (ended_at, duration) = match action.track_type {
            ActionTrackType::TimeSpan => (None, None),
            ActionTrackType::Count => (Some(started_at), Some(0)),
        };

        let action_track = self
            .db
            .transaction(|txn| {
                Box::pin(async move {
                    let action_track = ActiveModel {
                        id: Set(uuid::Uuid::now_v7()),
                        user_id: Set(params.user.id),
                        action_id: Set(params.action_id),
                        started_at: Set(started_at),
                        ended_at: Set(ended_at),
                        duration: Set(duration),
                    }
                    .insert(txn)
                    .await
                    .map_err(handle_unique_constraint)?;

                    let should_update_first_track_at = params.user.first_track_at.is_none_or(|t| t > started_at);
                    if should_update_first_track_at {
                        update_user_first_track_at(txn, params.user, Some(started_at)).await?;
                    }

                    Ok(action_track)
                })
            })
            .await?;

        Ok(action_track)
    }

    async fn update(&self, params: ActionTrackUpdateParams) -> Result<Model, ActionTrackServiceError> {
        let action_track = Entity::find_by_id(params.action_track_id)
            .filter(Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionTrackServiceError::ActionTrackNotFound())?;
        let action = action::Entity::find_by_id(params.action_id)
            .filter(action::Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionTrackServiceError::ActionNotFound())?;

        let started_at = params.started_at.trunc_subsecs(0);
        let ended_at = params.ended_at.and_then(|e| Some(e.trunc_subsecs(0)));

        let before_update = action_track.clone();
        let mut action_track = action_track.into_active_model();
        action_track.started_at = Set(started_at);
        action_track.ended_at = Set(ended_at);
        action_track.duration = Set(ended_at.and_then(|e| Some((e - started_at).num_seconds())));
        action_track.action_id = Set(action.id);

        let after_update = self
            .db
            .transaction(|txn| {
                Box::pin(async move {
                    let after_update = action_track.update(txn).await.map_err(handle_unique_constraint)?;

                    let should_update_first_track_at = match params.user.first_track_at {
                        Some(u) => {
                            (u == before_update.started_at && before_update.started_at != after_update.started_at)
                                || u > after_update.started_at
                        }
                        None => true,
                    };
                    if should_update_first_track_at {
                        let first_track_at = resolve_first_track_at(txn, &params.user).await?;
                        update_user_first_track_at(txn, params.user, first_track_at).await?;
                    }

                    Ok(after_update)
                })
            })
            .await?;

        Ok(after_update)
    }

    async fn delete(&self, params: ActionTrackDeleteParams) -> Result<(), ActionTrackServiceError> {
        let action_track = Entity::find_by_id(params.action_track_id)
            .filter(Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionTrackServiceError::ActionTrackNotFound())?;

        self.db
            .transaction(|txn| {
                Box::pin(async move {
                    action_track.clone().delete(txn).await?;

                    let should_update_first_track_at =
                        params.user.first_track_at.is_some_and(|t| t == action_track.started_at);
                    if should_update_first_track_at {
                        let first_track_at = resolve_first_track_at(txn, &params.user).await?;
                        update_user_first_track_at(txn, params.user, first_track_at).await?;
                    }

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
) -> Result<Option<DateTime<FixedOffset>>, ActionTrackServiceError> {
    Entity::find()
        .select_only()
        .column(Column::StartedAt)
        .filter(Column::UserId.eq(user.id))
        .join(LeftJoin, Relation::Action.def())
        .filter(action::Column::Archived.eq(false))
        .order_by_asc(Column::StartedAt)
        .into_tuple()
        .one(db)
        .await
        .map_err(|e| e.into())
}

async fn update_user_first_track_at(
    db: &impl ConnectionTrait,
    user: user::Model,
    first_track_at: Option<DateTime<FixedOffset>>,
) -> Result<user::Model, ActionTrackServiceError> {
    let mut user = user.into_active_model();
    user.first_track_at = Set(first_track_at);
    user.updated_at = Set(Utc::now().into());
    user.update(db).await.map_err(|e| e.into())
}
