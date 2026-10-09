use std::future::Future;

use chrono::{DateTime, FixedOffset, SubsecRound};
use entities::{
    action::{self, ActionTrackType},
    action_track::{ActiveModel, Column, Entity, Model},
    user,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde::Serialize;
use uuid::Uuid;

use crate::action_track_service::{handle_unique_constraint, ActionTrackService, ActionTrackServiceError};

#[derive(Serialize, Debug, Clone)]
pub struct ActionTrackCreateParams<'a> {
    pub user: &'a user::Model,
    pub action_id: Uuid,
    pub started_at: DateTime<FixedOffset>,
}
#[derive(Serialize, Debug, Clone)]
pub struct ActionTrackUpdateParams<'a> {
    pub user: &'a user::Model,
    pub action_track_id: Uuid,
    pub action_id: Uuid,
    pub started_at: DateTime<FixedOffset>,
    pub ended_at: Option<DateTime<FixedOffset>>,
}
pub trait ActionTrackServiceMutation {
    fn create(
        &self,
        params: ActionTrackCreateParams,
    ) -> impl Future<Output = Result<Model, ActionTrackServiceError>>;
    fn update(
        &self,
        params: ActionTrackUpdateParams,
    ) -> impl Future<Output = Result<(Model, Model), ActionTrackServiceError>>;
}

impl ActionTrackServiceMutation for ActionTrackService<'_> {
    async fn create(&self, params: ActionTrackCreateParams<'_>) -> Result<Model, ActionTrackServiceError> {
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

        let action_track = ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(params.user.id),
            action_id: Set(params.action_id),
            started_at: Set(started_at),
            ended_at: Set(ended_at),
            duration: Set(duration),
        }
        .insert(self.db)
        .await
        .map_err(handle_unique_constraint)?;

        Ok(action_track)
    }

    async fn update(
        &self,
        params: ActionTrackUpdateParams<'_>,
    ) -> Result<(Model, Model), ActionTrackServiceError> {
        let action_track = Entity::find_by_id(params.action_track_id)
            .filter(Column::UserId.eq(params.user.id))
            .one(self.db)
            .await?
            .ok_or(ActionTrackServiceError::ActionTrackNotFound())?;
        // MYMEMO: add test for this
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
        let after_update = action_track.update(self.db).await.map_err(handle_unique_constraint)?;

        Ok((after_update, before_update))
    }

    // async fn link_tags(&self, params: ActionTrackLinkTagsParams) -> Result<(), ActionTrackServiceError> {
    //     let note_exists = Entity::find_by_id(params.note_id)
    //         .filter(get_viewing_permission_filter(params.user_open_id))
    //         .exists(self.db)
    //         .await?;
    //     if !note_exists {
    //         return Err(ActionTrackServiceError::ActionTrackNotFound());
    //     }
    //     let tags = tag::Entity::find()
    //         .filter(tag::Column::UserId.eq(params.user_id))
    //         .filter(tag::Column::Id.is_in(params.tag_ids.clone()))
    //         .all(self.db)
    //         .await?;
    //     let currently_linked_tag_ids = tags_notes::Entity::find()
    //         .filter(tags_notes::Column::ActionTrackId.eq(params.note_id))
    //         .all(self.db)
    //         .await?
    //         .iter()
    //         .map(|link| link.tag_id)
    //         .collect::<Vec<_>>();
    //     let tag_ids_to_unlink = currently_linked_tag_ids
    //         .clone()
    //         .into_iter()
    //         .filter(|id| !params.tag_ids.contains(id))
    //         .collect::<Vec<_>>();

    //     self.db
    //         .transaction::<_, (), ActionTrackServiceError>(|txn| {
    //             Box::pin(async move {
    //                 for tag in tags {
    //                     let already_linked = currently_linked_tag_ids.contains(&tag.id);
    //                     if already_linked {
    //                         continue;
    //                     }
    //                     tags_notes::ActiveModel::builder()
    //                         .set_note_id(params.note_id)
    //                         .set_tag_id(tag.id)
    //                         .insert(txn)
    //                         .await?;
    //                 }
    //                 tags_notes::Entity::delete_many()
    //                     .filter(tags_notes::Column::ActionTrackId.eq(params.note_id))
    //                     .filter(tags_notes::Column::TagId.is_in(tag_ids_to_unlink))
    //                     .exec(txn)
    //                     .await?;

    //                 Ok(())
    //             })
    //         })
    //         .await?;

    //     Ok(())
    // }
}
