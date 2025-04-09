use entities::{ambition, tag};
use chrono::Utc;
use sea_orm::{prelude::*, ActiveValue::NotSet, DbConn, DbErr, Set};
use std::future::Future;
use uuid::Uuid;

pub fn ambition(user_id: Uuid) -> ambition::ActiveModel {
    let now = Utc::now();
    ambition::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        name: Set("ambition".to_string()),
        description: Set(None),
        archived: Set(false),
        ordering: NotSet,
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait AmbitionFactory {
    fn name(self, name: String) -> ambition::ActiveModel;
    fn description(self, description: Option<String>) -> ambition::ActiveModel;
    fn archived(self, archived: bool) -> ambition::ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> ambition::ActiveModel;
    fn insert_with_tag(
        self,
        db: &DbConn,
    ) -> impl Future<Output = Result<(ambition::Model, tag::Model), DbErr>> + Send;
}

impl AmbitionFactory for ambition::ActiveModel {
    fn name(mut self, name: String) -> ambition::ActiveModel {
        self.name = Set(name);
        self
    }

    fn description(mut self, description: Option<String>) -> ambition::ActiveModel {
        self.description = Set(description);
        self
    }

    fn archived(mut self, archived: bool) -> ambition::ActiveModel {
        self.archived = Set(archived);
        self
    }

    fn ordering(mut self, ordering: Option<i32>) -> ambition::ActiveModel {
        self.ordering = Set(ordering);
        self
    }

    async fn insert_with_tag(self, db: &DbConn) -> Result<(ambition::Model, tag::Model), DbErr> {
        let ambition = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(ambition.user_id),
            ambition_id: Set(Some(ambition.id)),
            desired_state_id: NotSet,
            action_id: NotSet,
            created_at: Set(Utc::now().into()),
        }
        .insert(db)
        .await?;
        Ok((ambition, tag))
    }
}
