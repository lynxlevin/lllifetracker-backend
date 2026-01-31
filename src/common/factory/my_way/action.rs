use chrono::Utc;
use entities::{
    action,
    sea_orm_active_enums::{ActionTrackType, TagType},
    tag,
};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, Set};
use std::future::Future;
use uuid::Uuid;

pub fn action(user_id: Uuid) -> action::ActiveModel {
    let now = Utc::now();
    action::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        name: Set("action".to_string()),
        discipline: Set(None),
        memo: Set(None),
        archived: Set(false),
        ordering: NotSet,
        color: Set("#212121".to_string()),
        track_type: Set(ActionTrackType::TimeSpan),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait ActionFactory {
    fn name(self, name: String) -> action::ActiveModel;
    fn discipline(self, discipline: Option<String>) -> action::ActiveModel;
    fn archived(self, archived: bool) -> action::ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> action::ActiveModel;
    fn track_type(self, track_type: ActionTrackType) -> action::ActiveModel;
    fn insert_with_tag(
        self,
        db: &DbConn,
    ) -> impl Future<Output = Result<(action::Model, tag::Model), DbErr>> + Send;
}

impl ActionFactory for action::ActiveModel {
    fn name(mut self, name: String) -> action::ActiveModel {
        self.name = Set(name);
        self
    }

    fn discipline(mut self, discipline: Option<String>) -> action::ActiveModel {
        self.discipline = Set(discipline);
        self
    }

    fn archived(mut self, archived: bool) -> action::ActiveModel {
        self.archived = Set(archived);
        self
    }

    fn ordering(mut self, ordering: Option<i32>) -> action::ActiveModel {
        self.ordering = Set(ordering);
        self
    }

    fn track_type(mut self, track_type: ActionTrackType) -> action::ActiveModel {
        self.track_type = Set(track_type);
        self
    }

    async fn insert_with_tag(self, db: &DbConn) -> Result<(action::Model, tag::Model), DbErr> {
        let action = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(action.user_id),
            action_id: Set(Some(action.id)),
            r#type: Set(TagType::Action),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok((action, tag))
    }
}
