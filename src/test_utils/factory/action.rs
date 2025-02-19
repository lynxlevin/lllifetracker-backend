use crate::entities::{action, tag};
use chrono::Utc;
use sea_orm::{prelude::*, ActiveValue::NotSet, DbConn, DbErr, Set};
use std::future::Future;
use uuid::Uuid;

#[cfg(test)]
pub fn action(user_id: Uuid) -> action::ActiveModel {
    let now = Utc::now();
    action::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set("action".to_string()),
        description: Set(None),
        archived: Set(false),
        ordering: NotSet,
        trackable: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

#[cfg(test)]
pub trait ActionFactory {
    fn name(self, name: String) -> action::ActiveModel;
    fn description(self, description: Option<String>) -> action::ActiveModel;
    fn archived(self, archived: bool) -> action::ActiveModel;
    fn insert_with_tag(
        self,
        db: &DbConn,
    ) -> impl Future<Output = Result<(action::Model, tag::Model), DbErr>> + Send;
}

#[cfg(test)]
impl ActionFactory for action::ActiveModel {
    fn name(mut self, name: String) -> action::ActiveModel {
        self.name = Set(name);
        self
    }

    fn description(mut self, description: Option<String>) -> action::ActiveModel {
        self.description = Set(description);
        self
    }

    fn archived(mut self, archived: bool) -> action::ActiveModel {
        self.archived = Set(archived);
        self
    }

    async fn insert_with_tag(self, db: &DbConn) -> Result<(action::Model, tag::Model), DbErr> {
        let action = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set(action.user_id),
            ambition_id: NotSet,
            objective_id: NotSet,
            action_id: Set(Some(action.id)),
            created_at: Set(Utc::now().into()),
        }
        .insert(db)
        .await?;
        Ok((action, tag))
    }
}
