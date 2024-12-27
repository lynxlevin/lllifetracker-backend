use crate::entities::{ambition, tag};
use chrono::Utc;
use sea_orm::{prelude::*, ActiveValue::NotSet, DbConn, DbErr, Set};
use uuid::Uuid;

#[cfg(test)]
pub fn ambition(user_id: Uuid) -> ambition::ActiveModel {
    ambition::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set("ambition".to_string()),
        description: Set(None),
        archived: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
}

#[cfg(test)]
impl ambition::ActiveModel {
    pub fn name(mut self, name: String) -> ambition::ActiveModel {
        self.name = Set(name);
        self
    }

    pub fn description(mut self, description: Option<String>) -> ambition::ActiveModel {
        self.description = Set(description);
        self
    }

    pub fn archived(mut self, archived: bool) -> ambition::ActiveModel {
        self.archived = Set(archived);
        self
    }

    pub async fn insert_with_tag(self, db: &DbConn) -> Result<(ambition::Model, tag::Model), DbErr> {
        let ambition = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set(ambition.user_id),
            ambition_id: Set(Some(ambition.id)),
            objective_id: NotSet,
            action_id: NotSet,
            created_at: Set(Utc::now().into()),
        }
        .insert(db)
        .await?;
        Ok((ambition, tag))
    }
}
