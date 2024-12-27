use crate::entities::{objective, tag};
use chrono::Utc;
use sea_orm::{prelude::*, ActiveValue::NotSet, DbConn, DbErr, Set};
use uuid::Uuid;

#[cfg(test)]
pub fn objective(user_id: Uuid) -> objective::ActiveModel {
    objective::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set("objective".to_string()),
        description: Set(None),
        archived: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
}

#[cfg(test)]
impl objective::ActiveModel {
    pub fn name(mut self, name: String) -> objective::ActiveModel {
        self.name = Set(name);
        self
    }

    pub fn description(mut self, description: Option<String>) -> objective::ActiveModel {
        self.description = Set(description);
        self
    }

    pub fn archived(mut self, archived: bool) -> objective::ActiveModel {
        self.archived = Set(archived);
        self
    }

    pub async fn insert_with_tag(
        self,
        db: &DbConn,
    ) -> Result<(objective::Model, tag::Model), DbErr> {
        let objective = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set(objective.user_id),
            ambition_id: NotSet,
            objective_id: Set(Some(objective.id)),
            action_id: NotSet,
            created_at: Set(Utc::now().into()),
        }
        .insert(db)
        .await?;
        Ok((objective, tag))
    }
}
