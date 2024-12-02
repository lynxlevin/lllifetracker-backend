use crate::entities::{objective, objectives_actions, tag};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{Set, TransactionError, TransactionTrait};

use super::objective_query::ObjectiveQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewObjective {
    pub name: String,
    pub description: Option<String>,
    pub user_id: uuid::Uuid,
}

pub struct ObjectiveMutation;

impl ObjectiveMutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewObjective,
    ) -> Result<objective::Model, TransactionError<DbErr>> {
        db.transaction::<_, objective::Model, DbErr>(|txn| {
            Box::pin(async move {
                let now = Utc::now();
                let objective_id = uuid::Uuid::new_v4();
                let created_objective = objective::ActiveModel {
                    id: Set(objective_id),
                    user_id: Set(form_data.user_id),
                    name: Set(form_data.name.to_owned()),
                    description: Set(form_data.description.to_owned()),
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    user_id: Set(form_data.user_id),
                    ambition_id: NotSet,
                    objective_id: Set(Some(objective_id)),
                    action_id: NotSet,
                    created_at: Set(now.into()),
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
        description: Option<String>,
    ) -> Result<objective::Model, DbErr> {
        let mut objective: objective::ActiveModel =
            ObjectiveQuery::find_by_id_and_user_id(db, objective_id, user_id)
                .await?
                .into();
        objective.name = Set(name);
        objective.description = Set(description);
        objective.updated_at = Set(Utc::now().into());
        objective.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        objective_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        ObjectiveQuery::find_by_id_and_user_id(db, objective_id, user_id)
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

#[cfg(test)]
mod tests {
    use crate::{test_utils, types::CustomDbErr};

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let name = "create_with_tag".to_string();
        let description = "Create with tag.".to_string();

        let form_data = NewObjective {
            name: name.clone(),
            description: Some(description.clone()),
            user_id: user.id,
        };
        let returned_objective = ObjectiveMutation::create_with_tag(&db, form_data)
            .await
            .unwrap();
        assert_eq!(returned_objective.name, name.clone());
        assert_eq!(returned_objective.description, Some(description.clone()));
        assert_eq!(returned_objective.user_id, user.id);

        let created_objective = objective::Entity::find_by_id(returned_objective.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_objective.name, returned_objective.name);
        assert_eq!(
            created_objective.description,
            returned_objective.description
        );
        assert_eq!(created_objective.user_id, returned_objective.user_id);
        assert_eq!(created_objective.created_at, returned_objective.created_at);
        assert_eq!(created_objective.updated_at, returned_objective.updated_at);

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
            None,
            user.id,
        )
        .await?;
        let new_name = "objective_after_update".to_string();
        let new_description = "Objective after update.".to_string();

        let returned_objective =
            ObjectiveMutation::update(&db, objective.id, user.id, new_name.clone(), Some(new_description.clone())).await?;
        assert_eq!(returned_objective.id, objective.id);
        assert_eq!(returned_objective.name, new_name.clone());
        assert_eq!(returned_objective.description, Some(new_description.clone()));
        assert_eq!(returned_objective.user_id, user.id);
        assert_eq!(returned_objective.created_at, objective.created_at);
        assert!(returned_objective.updated_at > objective.updated_at);

        let updated_objective = objective::Entity::find_by_id(objective.id)
            .one(&db)
            .await?.unwrap();
        assert_eq!(updated_objective.name, new_name.clone());
        assert_eq!(updated_objective.description, Some(new_description.clone()));
        assert_eq!(updated_objective.user_id, user.id);
        assert_eq!(updated_objective.created_at, objective.created_at);
        assert_eq!(updated_objective.updated_at, returned_objective.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, _) = test_utils::seed::create_objective_and_tag(
            &db,
            "objective_before_update_unauthorized".to_string(),
            None,
            user.id,
        )
        .await?;
        let new_name = "objective_after_update_unauthorized".to_string();

        let error =
            ObjectiveMutation::update(&db, objective.id, uuid::Uuid::new_v4(), new_name.clone(), None)
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
            None,
            user.id,
        )
        .await?;

        ObjectiveMutation::delete(&db, objective.id, user.id).await?;

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
            None,
            user.id,
        )
        .await?;

        let error = ObjectiveMutation::delete(&db, objective.id, uuid::Uuid::new_v4())
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
            "Test objective_service::ObjectiveMutation::connect_action".to_string(),
            None,
            user.id,
        )
        .await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "Test objective_service::ObjectiveMutation::connect_action".to_string(),
            None,
            user.id,
        )
        .await?;

        ObjectiveMutation::connect_action(&db, objective.id, action.id).await?;

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
            "Test objective_service::ObjectiveMutation::disconnect_action".to_string(),
            None,
            user.id,
        )
        .await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "Test objective_service::ObjectiveMutation::disconnect_action".to_string(),
            None,
            user.id,
        )
        .await?;
        let _connection = objectives_actions::ActiveModel {
            objective_id: Set(objective.id),
            action_id: Set(action.id),
        }
        .insert(&db)
        .await?;

        ObjectiveMutation::disconnect_action(&db, objective.id, action.id).await?;

        let connection_in_db = objectives_actions::Entity::find()
            .filter(objectives_actions::Column::ObjectiveId.eq(objective.id))
            .filter(objectives_actions::Column::ActionId.eq(action.id))
            .one(&db)
            .await?;
        assert!(connection_in_db.is_none());

        Ok(())
    }
}
