use crate::entities::{ambition, ambitions_objectives, tag};
use crate::types::CustomDbErr;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryOrder, Set, TransactionError, TransactionTrait};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewAmbition {
    pub name: String,
    pub description: Option<String>,
    pub user_id: uuid::Uuid,
}

pub struct Mutation;

impl Mutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewAmbition,
    ) -> Result<ambition::Model, TransactionError<DbErr>> {
        db.transaction::<_, ambition::Model, DbErr>(|txn| {
            Box::pin(async move {
                let ambition_id = uuid::Uuid::new_v4();
                let created_ambition = ambition::ActiveModel {
                    id: Set(ambition_id),
                    user_id: Set(form_data.user_id),
                    name: Set(form_data.name.to_owned()),
                    description: Set(form_data.description),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    user_id: Set(form_data.user_id),
                    ambition_id: Set(Some(ambition_id)),
                    ..Default::default()
                }
                .insert(txn)
                .await?;

                Ok(created_ambition)
            })
        })
        .await
    }

    pub async fn update(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<ambition::Model, DbErr> {
        let mut ambition: ambition::ActiveModel =
            Query::find_by_id_and_user_id(db, ambition_id, user_id)
                .await?
                .into();
        ambition.name = Set(name);
        ambition.description = Set(description);
        ambition.updated_at = Set(Utc::now().into());
        ambition.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        Query::find_by_id_and_user_id(db, ambition_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn connect_objective(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        objective_id: uuid::Uuid,
    ) -> Result<ambitions_objectives::Model, DbErr> {
        ambitions_objectives::ActiveModel {
            ambition_id: Set(ambition_id),
            objective_id: Set(objective_id),
        }
        .insert(db)
        .await
    }

    pub async fn disconnect_objective(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        objective_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        match ambitions_objectives::Entity::find()
            .filter(ambitions_objectives::Column::AmbitionId.eq(ambition_id))
            .filter(ambitions_objectives::Column::ObjectiveId.eq(objective_id))
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
    ) -> Result<Vec<ambition::Model>, DbErr> {
        ambition::Entity::find()
            .filter(ambition::Column::UserId.eq(user_id))
            .order_by_asc(ambition::Column::CreatedAt)
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<ambition::Model, DbErr> {
        ambition::Entity::find_by_id(ambition_id)
            .filter(ambition::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}
