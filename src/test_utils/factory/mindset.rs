use chrono::Utc;
use entities::{mindset, tag};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, Set};
use std::future::Future;
use uuid::Uuid;

pub fn mindset(user_id: Uuid) -> mindset::ActiveModel {
    let now = Utc::now();
    mindset::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        name: Set("mindset".to_string()),
        description: Set(None),
        archived: Set(false),
        ordering: NotSet,
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait MindsetFactory {
    fn name(self, name: String) -> mindset::ActiveModel;
    fn description(self, description: Option<String>) -> mindset::ActiveModel;
    fn archived(self, archived: bool) -> mindset::ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> mindset::ActiveModel;
    fn insert_with_tag(
        self,
        db: &DbConn,
    ) -> impl Future<Output = Result<(mindset::Model, tag::Model), DbErr>> + Send;
}

impl MindsetFactory for mindset::ActiveModel {
    fn name(mut self, name: String) -> mindset::ActiveModel {
        self.name = Set(name);
        self
    }

    fn description(mut self, description: Option<String>) -> mindset::ActiveModel {
        self.description = Set(description);
        self
    }

    fn archived(mut self, archived: bool) -> mindset::ActiveModel {
        self.archived = Set(archived);
        self
    }

    fn ordering(mut self, ordering: Option<i32>) -> mindset::ActiveModel {
        self.ordering = Set(ordering);
        self
    }

    async fn insert_with_tag(self, db: &DbConn) -> Result<(mindset::Model, tag::Model), DbErr> {
        let mindset = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(mindset.user_id),
            mindset_id: Set(Some(mindset.id)),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok((mindset, tag))
    }
}
