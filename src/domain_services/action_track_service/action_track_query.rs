use std::future::Future;

use chrono::{DateTime, FixedOffset};
use entities::{
    action,
    action_track::{Column, Entity, Model, Relation},
};
use sea_orm::{ColumnTrait, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::action_track_service::{ActionTrackService, ActionTrackServiceError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionTrackListParams {
    pub active_only: Option<bool>,
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
}
pub trait ActionTrackServiceQuery {
    fn list(
        &self,
        user_id: Uuid,
        params: ActionTrackListParams,
    ) -> impl Future<Output = Result<Vec<Model>, ActionTrackServiceError>>;
}

impl ActionTrackServiceQuery for ActionTrackService<'_> {
    async fn list(
        &self,
        user_id: Uuid,
        params: ActionTrackListParams,
    ) -> Result<Vec<Model>, ActionTrackServiceError> {
        let mut query = Entity::find()
            .join(LeftJoin, Relation::Action.def())
            .filter(Column::UserId.eq(user_id))
            .filter(action::Column::Archived.eq(false));
        if params.active_only.is_some_and(|active_only| active_only) {
            query = query.filter(Column::EndedAt.is_null());
        }
        if let Some(started_at_gte) = params.started_at_gte {
            query = query.filter(Column::StartedAt.gte(started_at_gte));
        }
        if let Some(started_at_lte) = params.started_at_lte {
            query = query.filter(Column::StartedAt.lte(started_at_lte));
        }
        let action_tracks = query.order_by_desc(Column::StartedAt).all(self.db).await?;

        Ok(action_tracks)
    }
}
