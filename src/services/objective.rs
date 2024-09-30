use crate::entities::{objective, objectives_actions, tag};
use crate::types::CustomDbErr;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryOrder, Set, TransactionError, TransactionTrait};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewObjective {
    pub name: String,
    pub user_id: uuid::Uuid,
}

pub struct Mutation;

impl Mutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewObjective,
    ) -> Result<objective::Model, TransactionError<DbErr>> {
        db.transaction::<_, objective::Model, DbErr>(|txn| {
            Box::pin(async move {
                let objective_id = uuid::Uuid::new_v4();
                let created_objective = objective::ActiveModel {
                    id: Set(objective_id),
                    user_id: Set(form_data.user_id),
                    name: Set(form_data.name.to_owned()),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    user_id: Set(form_data.user_id),
                    objective_id: Set(Some(objective_id)),
                    ..Default::default()
                }
                .insert(txn)
                .await?;

                Ok(created_objective)
            })
        })
        .await
    }

    pub async fn update(
        db: &DbConn,
        objective_id: uuid::Uuid,
        user_id: uuid::Uuid,
        name: String,
    ) -> Result<objective::Model, DbErr> {
        let mut objective: objective::ActiveModel =
            Query::find_by_id_and_user_id(db, objective_id, user_id)
                .await?
                .into();
        objective.name = Set(name);
        objective.updated_at = Set(Utc::now().into());
        objective.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        objective_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        Query::find_by_id_and_user_id(db, objective_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn connect_action(
        db: &DbConn,
        objective_id: uuid::Uuid,
        action_id: uuid::Uuid,
    ) -> Result<objectives_actions::Model, DbErr> {
        objectives_actions::ActiveModel {
            objective_id: Set(objective_id),
            action_id: Set(action_id),
        }
        .insert(db)
        .await
    }

    pub async fn disconnect_action(
        db: &DbConn,
        objective_id: uuid::Uuid,
        action_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        match objectives_actions::Entity::find()
            .filter(objectives_actions::Column::ObjectiveId.eq(objective_id))
            .filter(objectives_actions::Column::ActionId.eq(action_id))
            .one(db)
            .await
        {
            Ok(connection) => match connection {
                Some(connection) => {
                    connection.delete(db).await?;
                    Ok(())
                }
                None => Ok(()),
            },
            Err(e) => Err(e),
        }
    }
}

pub struct Query;

impl Query {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<objective::Model>, DbErr> {
        objective::Entity::find()
            .filter(objective::Column::UserId.eq(user_id))
            .order_by_asc(objective::Column::CreatedAt)
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        objective_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<objective::Model, DbErr> {
        objective::Entity::find_by_id(objective_id)
            .filter(objective::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}
