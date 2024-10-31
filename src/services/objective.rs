use crate::entities::{objective, objectives_actions, tag};
use crate::types::{CustomDbErr, ObjectiveVisible};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::NotSet;
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
                    created_at: Set(Utc::now().into()),
                    updated_at: Set(Utc::now().into()),
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    user_id: Set(form_data.user_id),
                    ambition_id: NotSet,
                    objective_id: Set(Some(objective_id)),
                    action_id: NotSet,
                    created_at: Set(Utc::now().into()),
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
    ) -> Result<Vec<ObjectiveVisible>, DbErr> {
        objective::Entity::find()
            .filter(objective::Column::UserId.eq(user_id))
            .order_by_asc(objective::Column::CreatedAt)
            .into_partial_model::<ObjectiveVisible>()
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

#[cfg(test)]
mod mutation_tests {
    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let name = "objective_service::Mutation::create_with_tag".to_string();

        let form_data = NewObjective {
            name: name.clone(),
            user_id: user.id,
        };
        let returned_objective = Mutation::create_with_tag(&db, form_data).await.unwrap();
        assert_eq!(returned_objective.name, name);
        assert_eq!(returned_objective.user_id, user.id);

        let created_objective = objective::Entity::find_by_id(returned_objective.id)
            .filter(objective::Column::Name.eq(returned_objective.name))
            .filter(objective::Column::UserId.eq(returned_objective.user_id))
            .filter(objective::Column::CreatedAt.eq(returned_objective.created_at))
            .filter(objective::Column::UpdatedAt.eq(returned_objective.updated_at))
            .one(&db)
            .await?;
        assert!(created_objective.is_some());

        let created_tag = tag::Entity::find()
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::ObjectiveId.eq(returned_objective.id))
            .filter(tag::Column::ActionId.is_null())
            .filter(tag::Column::UserId.eq(user.id))
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, _) = test_utils::seed::create_objective_and_tag(
            &db,
            "objective_before_update".to_string(),
            user.id,
        )
        .await?;
        let new_name = "objective_after_update".to_string();

        let returned_objective =
            Mutation::update(&db, objective.id, user.id, new_name.clone()).await?;
        assert_eq!(returned_objective.id, objective.id);
        assert_eq!(returned_objective.name, new_name.clone());
        assert_eq!(returned_objective.user_id, user.id);
        assert_eq!(returned_objective.created_at, objective.created_at);
        assert!(returned_objective.updated_at > objective.updated_at);

        let updated_objective = objective::Entity::find_by_id(objective.id)
            .filter(objective::Column::Name.eq(new_name))
            .filter(objective::Column::UserId.eq(user.id))
            .filter(objective::Column::CreatedAt.eq(objective.created_at))
            .filter(objective::Column::UpdatedAt.eq(returned_objective.updated_at))
            .one(&db)
            .await?;
        assert!(updated_objective.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, _) = test_utils::seed::create_objective_and_tag(
            &db,
            "objective_before_update_unauthorized".to_string(),
            user.id,
        )
        .await?;
        let new_name = "objective_after_update_unauthorized".to_string();

        let error = Mutation::update(&db, objective.id, uuid::Uuid::new_v4(), new_name.clone())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, tag) = test_utils::seed::create_objective_and_tag(
            &db,
            "objective_for_delete".to_string(),
            user.id,
        )
        .await?;

        Mutation::delete(&db, objective.id, user.id).await?;

        let objective_in_db = objective::Entity::find_by_id(objective.id).one(&db).await?;
        assert!(objective_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, _) = test_utils::seed::create_objective_and_tag(
            &db,
            "objective_for_delete_unauthorized".to_string(),
            user.id,
        )
        .await?;

        let error = Mutation::delete(&db, objective.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn connect_action() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, _) = test_utils::seed::create_objective_and_tag(
            &db,
            "Test objective_service::Mutation::connect_action".to_string(),
            user.id,
        )
        .await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "Test objective_service::Mutation::connect_action".to_string(),
            user.id,
        )
        .await?;

        Mutation::connect_action(&db, objective.id, action.id).await?;

        let created_connection = objectives_actions::Entity::find()
            .filter(objectives_actions::Column::ObjectiveId.eq(objective.id))
            .filter(objectives_actions::Column::ActionId.eq(action.id))
            .one(&db)
            .await?;
        assert!(created_connection.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn disconnect_action() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, _) = test_utils::seed::create_objective_and_tag(
            &db,
            "Test objective_service::Mutation::disconnect_action".to_string(),
            user.id,
        )
        .await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "Test objective_service::Mutation::disconnect_action".to_string(),
            user.id,
        )
        .await?;
        let _connection = objectives_actions::ActiveModel {
            objective_id: Set(objective.id),
            action_id: Set(action.id),
        }
        .insert(&db)
        .await?;

        Mutation::disconnect_action(&db, objective.id, action.id).await?;

        let connection_in_db = objectives_actions::Entity::find()
            .filter(objectives_actions::Column::ObjectiveId.eq(objective.id))
            .filter(objectives_actions::Column::ActionId.eq(action.id))
            .one(&db)
            .await?;
        assert!(connection_in_db.is_none());

        Ok(())
    }
}
