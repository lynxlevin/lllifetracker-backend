use crate::entities::{ambition, ambitions_objectives, tag};
use crate::types::CustomDbErr;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::NotSet;
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
                    created_at: Set(Utc::now().into()),
                    updated_at: Set(Utc::now().into()),
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    user_id: Set(form_data.user_id),
                    ambition_id: Set(Some(ambition_id)),
                    objective_id: NotSet,
                    action_id: NotSet,
                    created_at: Set(Utc::now().into()),
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

#[cfg(test)]
mod mutation_tests {
    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let name = "Test ambition_service::Mutation::create_with_tag".to_string();
        let description = Some("Dummy description".to_string());

        let form_data = NewAmbition {
            name: name.clone(),
            description: description.clone(),
            user_id: user.id,
        };

        let returned_ambition = Mutation::create_with_tag(&db, form_data).await.unwrap();
        assert_eq!(returned_ambition.name, name);
        assert_eq!(returned_ambition.description, description);
        assert_eq!(returned_ambition.user_id, user.id);

        let created_ambition = ambition::Entity::find_by_id(returned_ambition.id)
            .filter(ambition::Column::Name.eq(name))
            .filter(ambition::Column::Description.eq(description))
            .filter(ambition::Column::UserId.eq(user.id))
            .filter(ambition::Column::CreatedAt.eq(returned_ambition.created_at))
            .filter(ambition::Column::UpdatedAt.eq(returned_ambition.updated_at))
            .one(&db)
            .await?;
        assert!(created_ambition.is_some());

        let created_tag = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::AmbitionId.eq(Some(returned_ambition.id)))
            .filter(tag::Column::ObjectiveId.is_null())
            .filter(tag::Column::ActionId.is_null())
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (ambition, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "Test ambition_service::Mutation::update".to_string(),
            None,
            user.id,
        )
        .await?;
        let new_name = "Test ambition_service::Mutation::update_after".to_string();
        let new_description = Some("After update.".to_string());

        let returned_ambition = Mutation::update(
            &db,
            ambition.id,
            user.id,
            new_name.clone(),
            new_description.clone(),
        )
        .await?;
        assert_eq!(returned_ambition.id, ambition.id);
        assert_eq!(returned_ambition.name, new_name);
        assert_eq!(returned_ambition.description, new_description);
        assert_eq!(returned_ambition.user_id, user.id);
        assert_eq!(returned_ambition.created_at, ambition.created_at);
        assert!(returned_ambition.updated_at > ambition.updated_at);

        let updated_ambition = ambition::Entity::find_by_id(returned_ambition.id)
            .filter(ambition::Column::Name.eq(new_name))
            .filter(ambition::Column::Description.eq(new_description))
            .filter(ambition::Column::UserId.eq(user.id))
            .filter(ambition::Column::CreatedAt.eq(returned_ambition.created_at))
            .filter(ambition::Column::UpdatedAt.eq(returned_ambition.updated_at))
            .one(&db)
            .await?;
        assert!(updated_ambition.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (ambition, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "Test ambition_service::Mutation::update".to_string(),
            None,
            user.id,
        )
        .await?;
        let new_name = "Test ambition_service::Mutation::update_after".to_string();
        let new_description = Some("After update.".to_string());

        let error = Mutation::update(
            &db,
            ambition.id,
            uuid::Uuid::new_v4(),
            new_name.clone(),
            new_description.clone(),
        )
        .await
        .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (ambition, tag) = test_utils::seed::create_ambition_and_tag(
            &db,
            "Test ambition_service::Mutation::delete".to_string(),
            None,
            user.id,
        )
        .await?;

        Mutation::delete(&db, ambition.id, user.id).await?;

        let ambition_in_db = ambition::Entity::find_by_id(ambition.id).one(&db).await?;
        assert!(ambition_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (ambition, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "Test ambition_service::Mutation::delete_unauthorized".to_string(),
            None,
            user.id,
        )
        .await?;

        let error = Mutation::delete(&db, ambition.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn connect_objective() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (ambition, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "Test ambition_service::Mutation::connect_objective".to_string(),
            None,
            user.id,
        )
        .await?;
        let (objective, _) = test_utils::seed::create_objective_and_tag(
            &db,
            "Test ambition_service::Mutation::connect_objective".to_string(),
            user.id,
        )
        .await?;

        Mutation::connect_objective(&db, ambition.id, objective.id).await?;

        let created_connection = ambitions_objectives::Entity::find()
            .filter(ambitions_objectives::Column::AmbitionId.eq(ambition.id))
            .filter(ambitions_objectives::Column::ObjectiveId.eq(objective.id))
            .one(&db)
            .await?;
        assert!(created_connection.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn disconnect_objective() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (ambition, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "Test ambition_service::Mutation::disconnect_objective".to_string(),
            None,
            user.id,
        )
        .await?;
        let (objective, _) = test_utils::seed::create_objective_and_tag(
            &db,
            "Test ambition_service::Mutation::disconnect_objective".to_string(),
            user.id,
        )
        .await?;
        let _connection = ambitions_objectives::ActiveModel {
            ambition_id: Set(ambition.id),
            objective_id: Set(objective.id),
        }
        .insert(&db)
        .await?;

        Mutation::disconnect_objective(&db, ambition.id, objective.id).await?;

        let connection_in_db = ambitions_objectives::Entity::find()
            .filter(ambitions_objectives::Column::AmbitionId.eq(ambition.id))
            .filter(ambitions_objectives::Column::ObjectiveId.eq(objective.id))
            .one(&db)
            .await?;
        assert!(connection_in_db.is_none());

        Ok(())
    }
}
