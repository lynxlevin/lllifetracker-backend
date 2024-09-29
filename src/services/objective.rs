use crate::entities::{objective, tag};
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
    ) -> Option<Result<objective::Model, DbErr>> {
        match Query::find_by_id_and_user_id(db, objective_id, user_id).await {
            Ok(objective) => match objective {
                Some(objective) => {
                    let mut objective: objective::ActiveModel = objective.into();
                    objective.name = Set(name);
                    objective.updated_at = Set(Utc::now().into());
                    Some(objective.update(db).await)
                }
                None => None,
            },
            Err(e) => Some(Err(e)),
        }
    }

    pub async fn delete(
        db: &DbConn,
        objective_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        match Query::find_by_id_and_user_id(db, objective_id, user_id).await {
            Ok(objective) => match objective {
                Some(objective) => {
                    objective.delete(db).await?;
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
    ) -> Result<Option<objective::Model>, DbErr> {
        objective::Entity::find_by_id(objective_id)
            .filter(objective::Column::UserId.eq(user_id))
            .one(db)
            .await
    }
}
