use crate::entities::{action, tag};
use chrono::Utc;
use sea_orm::{prelude::*, ActiveValue::NotSet, DbConn, DbErr, Set};
use uuid::Uuid;

#[cfg(test)]
pub fn action(user_id: Uuid) -> action::ActiveModel {
    action::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set("action".to_string()),
        description: Set(None),
        archived: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
}

#[cfg(test)]
impl action::ActiveModel {
    pub fn name(mut self, name: String) -> action::ActiveModel {
        self.name = Set(name);
        self
    }

    pub fn description(mut self, description: Option<String>) -> action::ActiveModel {
        self.description = Set(description);
        self
    }

    pub fn archived(mut self, archived: bool) -> action::ActiveModel {
        self.archived = Set(archived);
        self
    }

    pub async fn insert_with_tag(self, db: &DbConn) -> Result<(action::Model, tag::Model), DbErr> {
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
