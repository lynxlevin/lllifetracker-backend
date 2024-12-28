use sea_orm::{ActiveModelTrait, DbConn, DbErr, Set};

use crate::entities::{ambition, ambitions_objectives, objective, objectives_actions};

#[cfg(test)]
impl ambition::Model {
    pub async fn connect_objective(self, db: &DbConn, objective_id: uuid::Uuid) -> Result<ambition::Model, DbErr> {
        ambitions_objectives::ActiveModel {
            ambition_id: Set(self.id),
            objective_id: Set(objective_id),
        }
        .insert(db)
        .await?;
        Ok(self)
    }
}

#[cfg(test)]
impl objective::Model {
    pub async fn connect_action(self, db: &DbConn, action_id: uuid::Uuid) -> Result<objective::Model, DbErr> {
        objectives_actions::ActiveModel {
            objective_id: Set(self.id),
            action_id: Set(action_id),
        }
        .insert(db)
        .await?;
        Ok(self)
    }
}