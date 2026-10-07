use std::future::Future;

use chrono::Utc;
use entities::{
    note::{ActiveModel, Entity, Model},
    note_permission::{self, NotePermissionEnum},
    tag, tags_notes,
    user::{self, OpenId},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, SelectExt, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::note_service::{
    check_editor_permission, check_owner_permission, get_viewing_permission_filter, NoteService, NoteServiceError,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoteCreateParams<'a> {
    pub name: &'a str,
    pub text: &'a str,
    pub user_open_id: OpenId,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoteUpdateParams<'a> {
    pub name: &'a str,
    pub text: &'a str,
    pub user_id: Uuid,
    pub user_open_id: OpenId,
    pub note_id: Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoteUpdateIsSecretParams {
    pub user_id: Uuid,
    pub user_open_id: OpenId,
    pub note_id: Uuid,
    pub is_secret: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoteLockParams {
    pub user_id: Uuid,
    pub user_open_id: OpenId,
    pub note_id: Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoteShareParams {
    pub user_open_id: OpenId,
    pub note_id: Uuid,
    pub friend_open_id: OpenId,
    pub permission: NotePermissionEnum,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoteDeleteParams {
    pub user_open_id: OpenId,
    pub note_id: Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoteLinkTagsParams {
    pub user_id: Uuid,
    pub user_open_id: OpenId,
    pub note_id: Uuid,
    pub tag_ids: Vec<Uuid>,
}

pub trait NoteServiceMutation {
    fn create(
        &self,
        params: NoteCreateParams,
    ) -> impl Future<Output = Result<(Model, note_permission::Model), NoteServiceError>>;
    fn update(&self, params: NoteUpdateParams) -> impl Future<Output = Result<Model, NoteServiceError>>;
    fn update_is_secret(
        &self,
        params: NoteUpdateIsSecretParams,
    ) -> impl Future<Output = Result<Model, NoteServiceError>>;
    fn lock(&self, params: NoteLockParams) -> impl Future<Output = Result<Model, NoteServiceError>>;
    fn unlock(&self, params: NoteLockParams) -> impl Future<Output = Result<Model, NoteServiceError>>;
    fn share(
        &self,
        params: NoteShareParams,
    ) -> impl Future<Output = Result<(Model, Vec<note_permission::Model>), NoteServiceError>>;
    fn delete(&self, params: NoteDeleteParams) -> impl Future<Output = Result<(), NoteServiceError>>;
    fn link_tags(&self, params: NoteLinkTagsParams) -> impl Future<Output = Result<(), NoteServiceError>>;
}

impl NoteServiceMutation for NoteService<'_> {
    async fn create(
        &self,
        params: NoteCreateParams<'_>,
    ) -> Result<(Model, note_permission::Model), NoteServiceError> {
        let text_head = shorten_text(&params.text);
        let note = ActiveModel::builder()
            .set_name(params.name.to_string())
            .set_text(params.text.to_string())
            .set_text_head(text_head)
            .add_note_permission(
                note_permission::ActiveModel::builder()
                    .set_user_open_id(params.user_open_id)
                    .set_permission(NotePermissionEnum::Owner),
            )
            .insert(self.db)
            .await?;
        let note_permission: note_permission::Model = note
            .note_permissions
            .clone()
            .into_iter()
            .next()
            .ok_or(NoteServiceError::RelatedRecordNotFound(
                "NotePermission could not be retrieved from DB.".to_string(),
            ))?
            .into();
        Ok((note.into(), note_permission))
    }

    async fn update(&self, params: NoteUpdateParams<'_>) -> Result<Model, NoteServiceError> {
        check_editor_permission(params.user_open_id, params.note_id, self.db).await?;

        let note = Entity::find_by_id(params.note_id)
            .one(self.db)
            .await?
            .ok_or(NoteServiceError::NoteNotFound())?;
        if note.lock_owner_open_id.is_none_or(|id| id != params.user_open_id) {
            return Err(NoteServiceError::UserHasNoLock(params.user_open_id, params.note_id));
        }

        let mut note = note.into_active_model();
        note.name = Set(params.name.to_string());
        note.text = Set(params.text.to_string());
        note.text_head = Set(shorten_text(params.text));
        note.lock_owner_open_id = Set(None);
        note.updated_at = Set(Utc::now().into());
        note.update(self.db).await.map_err(|e| e.into())
    }

    async fn update_is_secret(&self, params: NoteUpdateIsSecretParams) -> Result<Model, NoteServiceError> {
        check_owner_permission(params.user_open_id, params.note_id, self.db).await?;
        let is_shared = note_permission::Entity::find()
            .filter(note_permission::Column::UserOpenId.ne(params.user_open_id))
            .filter(note_permission::Column::NoteId.eq(params.note_id))
            .count(self.db)
            .await?
            > 0;
        if is_shared {
            return Err(NoteServiceError::SharedWithOther(params.note_id));
        }

        let note = Entity::find_by_id(params.note_id)
            .one(self.db)
            .await?
            .ok_or(NoteServiceError::NoteNotFound())?;
        if note.lock_owner_open_id.is_some_and(|id| id != params.user_open_id) {
            return Err(NoteServiceError::UserHasNoLock(params.user_open_id, params.note_id));
        }

        let mut note = note.into_active_model();
        note.is_secret = Set(params.is_secret);
        note.lock_owner_open_id = Set(None);
        note.updated_at = Set(Utc::now().into());
        note.update(self.db).await.map_err(|e| e.into())
    }

    async fn lock(&self, params: NoteLockParams) -> Result<Model, NoteServiceError> {
        check_editor_permission(params.user_open_id, params.note_id, self.db).await?;

        let note = Entity::find_by_id(params.note_id)
            .one(self.db)
            .await?
            .ok_or(NoteServiceError::NoteNotFound())?;
        if note.lock_owner_open_id.is_some_and(|id| id != params.user_open_id) {
            return Err(NoteServiceError::LockedByOther(
                note.lock_owner_open_id.unwrap(),
                params.note_id,
            ));
        }
        if note.lock_owner_open_id.is_some_and(|id| id == params.user_open_id) {
            return Ok(note);
        }

        let mut note = note.into_active_model();
        note.lock_owner_open_id = Set(Some(params.user_open_id));
        note.update(self.db).await.map_err(|e| e.into())
    }

    async fn unlock(&self, params: NoteLockParams) -> Result<Model, NoteServiceError> {
        check_editor_permission(params.user_open_id, params.note_id, self.db).await?;

        let note = Entity::find_by_id(params.note_id)
            .one(self.db)
            .await?
            .ok_or(NoteServiceError::NoteNotFound())?;
        if note.lock_owner_open_id.is_some_and(|id| id != params.user_open_id) {
            return Err(NoteServiceError::LockedByOther(
                note.lock_owner_open_id.unwrap(),
                params.note_id,
            ));
        }
        if note.lock_owner_open_id.is_none() {
            return Ok(note);
        }

        let mut note = note.into_active_model();
        note.lock_owner_open_id = Set(None);
        note.update(self.db).await.map_err(|e| e.into())
    }

    async fn share(
        &self,
        params: NoteShareParams,
    ) -> Result<(Model, Vec<note_permission::Model>), NoteServiceError> {
        check_owner_permission(params.user_open_id, params.note_id, self.db).await?;

        let note = Entity::find_by_id(params.note_id)
            .one(self.db)
            .await?
            .ok_or(NoteServiceError::NoteNotFound())?;

        if note.is_secret {
            return Err(NoteServiceError::SecretNote(note.id));
        }

        let friend = user::Entity::find_by_open_id(params.friend_open_id)
            .one(self.db)
            .await?
            .ok_or(NoteServiceError::FriendNotFound())?;

        note_permission::ActiveModel {
            user_open_id: Set(friend.open_id),
            note_id: Set(note.id),
            permission: Set(params.permission),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        let note_permissions = note_permission::Entity::find()
            .filter(note_permission::Column::NoteId.eq(note.id))
            .order_by_asc(note_permission::Column::CreatedAt)
            .all(self.db)
            .await?;

        Ok((note, note_permissions))
    }

    async fn delete(&self, params: NoteDeleteParams) -> Result<(), NoteServiceError> {
        check_owner_permission(params.user_open_id, params.note_id, self.db).await?;

        let note = Entity::find_by_id(params.note_id)
            .one(self.db)
            .await?
            .ok_or(NoteServiceError::NoteNotFound())?;
        note.delete(self.db).await?;

        Ok(())
    }

    async fn link_tags(&self, params: NoteLinkTagsParams) -> Result<(), NoteServiceError> {
        let note_exists = Entity::find_by_id(params.note_id)
            .filter(get_viewing_permission_filter(params.user_open_id))
            .exists(self.db)
            .await?;
        if !note_exists {
            return Err(NoteServiceError::NoteNotFound());
        }
        let tags = tag::Entity::find()
            .filter(tag::Column::UserId.eq(params.user_id))
            .filter(tag::Column::Id.is_in(params.tag_ids.clone()))
            .all(self.db)
            .await?;
        let currently_linked_tag_ids = tags_notes::Entity::find()
            .filter(tags_notes::Column::NoteId.eq(params.note_id))
            .all(self.db)
            .await?
            .iter()
            .map(|link| link.tag_id)
            .collect::<Vec<_>>();
        let tag_ids_to_unlink = currently_linked_tag_ids
            .clone()
            .into_iter()
            .filter(|id| !params.tag_ids.contains(id))
            .collect::<Vec<_>>();

        self.db
            .transaction::<_, (), NoteServiceError>(|txn| {
                Box::pin(async move {
                    for tag in tags {
                        let already_linked = currently_linked_tag_ids.contains(&tag.id);
                        if already_linked {
                            continue;
                        }
                        tags_notes::ActiveModel::builder()
                            .set_note_id(params.note_id)
                            .set_tag_id(tag.id)
                            .insert(txn)
                            .await?;
                    }
                    tags_notes::Entity::delete_many()
                        .filter(tags_notes::Column::NoteId.eq(params.note_id))
                        .filter(tags_notes::Column::TagId.is_in(tag_ids_to_unlink))
                        .exec(txn)
                        .await?;

                    Ok(())
                })
            })
            .await?;

        Ok(())
    }
}

fn shorten_text(text: &str) -> String {
    let mut needs_ellipsis = false;
    let lines = text.lines();
    let mut res = String::new();
    for (i, line) in lines.enumerate() {
        if i > 4 {
            needs_ellipsis = true;
            break;
        };
        if i > 0 {
            res.push('\n');
        }
        res.push_str(line);
    }

    let res = res.chars().take(101);
    if needs_ellipsis || res.clone().count() > 100 {
        let mut res: String = res.take(97).collect();
        res.push_str("...");
        res
    } else {
        res.take(100).collect()
    }
}
