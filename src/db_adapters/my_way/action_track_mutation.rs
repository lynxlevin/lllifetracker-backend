use sea_orm::{
    sqlx::error::Error::Database, ActiveModelTrait, DbConn, DbErr, IntoActiveModel, ModelTrait,
    RuntimeErr::SqlxError, Set,
};
use serde::{Deserialize, Serialize};

use entities::action_track::{ActiveModel, Model};

use crate::CustomDbErr;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateActionTrackParams {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration: Option<i64>,
    pub action_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UpdateActionTrackParams {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duration: Option<i64>,
    pub action_id: uuid::Uuid,
}

#[derive(Clone)]
pub struct ActionTrackMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> ActionTrackMutation<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self { db }
    }
    pub async fn create(self, params: CreateActionTrackParams) -> Result<Model, DbErr> {
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

    pub async fn update(
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

    pub async fn delete(self, action_track: Model) -> Result<(), DbErr> {
        action_track.delete(self.db).await.map(|_| ())
    }
}
