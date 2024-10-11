use crate::entities::{action, tag};
use crate::types::CustomDbErr;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{QueryOrder, Set, TransactionError, TransactionTrait};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewAction {
    pub name: String,
    pub user_id: uuid::Uuid,
}

pub struct Mutation;

impl Mutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewAction,
    ) -> Result<action::Model, TransactionError<DbErr>> {
        db.transaction::<_, action::Model, DbErr>(|txn| {
            Box::pin(async move {
                let action_id = uuid::Uuid::new_v4();
                let created_action = action::ActiveModel {
                    id: Set(action_id),
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
                    objective_id: NotSet,
                    action_id: Set(Some(action_id)),
                    created_at: Set(Utc::now().into()),
                }
                .insert(txn)
                .await?;

                Ok(created_action)
            })
        })
        .await
    }

    pub async fn update(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
        name: String,
    ) -> Result<action::Model, DbErr> {
        let mut action: action::ActiveModel = Query::find_by_id_and_user_id(db, action_id, user_id)
            .await?
            .into();
        action.name = Set(name);
        action.updated_at = Set(Utc::now().into());
        action.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        Query::find_by_id_and_user_id(db, action_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }
}

pub struct Query;

impl Query {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<action::Model>, DbErr> {
        action::Entity::find()
            .filter(action::Column::UserId.eq(user_id))
            .order_by_asc(action::Column::CreatedAt)
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<action::Model, DbErr> {
        action::Entity::find_by_id(action_id)
            .filter(action::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod mutation_tests {
    use sea_orm::DbErr;

    use crate::entities::tag;
    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let action_name = "Test action_service::Mutation::create_with_tag".to_string();

        let form_data = NewAction {
            name: action_name.clone(),
            user_id: user.id,
        };

        let returned_action = Mutation::create_with_tag(&db, form_data).await.unwrap();
        assert_eq!(returned_action.name, action_name.clone());
        assert_eq!(returned_action.user_id, user.id);

        let created_action = action::Entity::find_by_id(returned_action.id)
            .filter(action::Column::Name.eq(action_name))
            .filter(action::Column::UserId.eq(user.id))
            .filter(action::Column::CreatedAt.eq(returned_action.created_at))
            .filter(action::Column::UpdatedAt.eq(returned_action.updated_at))
            .one(&db)
            .await?;
        assert!(created_action.is_some());

        let created_tag = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::ObjectiveId.is_null())
            .filter(tag::Column::ActionId.eq(returned_action.id))
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "action_before_update".to_string(),
            user.id,
        )
        .await?;
        let new_name = "action_after_update".to_string();

        let returned_action = Mutation::update(&db, action.id, user.id, new_name.clone()).await?;
        assert_eq!(returned_action.id, action.id);
        assert_eq!(returned_action.name, new_name.clone());
        assert_eq!(returned_action.user_id, user.id);
        assert_eq!(returned_action.created_at, action.created_at);
        assert!(returned_action.updated_at > action.updated_at);

        let updated_action = action::Entity::find_by_id(action.id)
            .filter(action::Column::Name.eq(new_name))
            .filter(action::Column::UserId.eq(user.id))
            .filter(action::Column::CreatedAt.eq(action.created_at))
            .filter(action::Column::UpdatedAt.eq(returned_action.updated_at))
            .one(&db)
            .await?;
        assert!(updated_action.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "action_before_update_unauthorized".to_string(),
            user.id,
        )
        .await?;
        let new_name = "action_after_update_unauthorized".to_string();

        let error = Mutation::update(&db, action.id, uuid::Uuid::new_v4(), new_name.clone())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (action, tag) =
            test_utils::seed::create_action_and_tag(&db, "action_for_delete".to_string(), user.id)
                .await?;

        Mutation::delete(&db, action.id, user.id).await?;

        let action_in_db = action::Entity::find_by_id(action.id).one(&db).await?;
        assert!(action_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_user(&db).await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "action_for_delete_unauthorized".to_string(),
            user.id,
        )
        .await?;

        let error = Mutation::delete(&db, action.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }
}
