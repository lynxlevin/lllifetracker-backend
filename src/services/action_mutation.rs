use crate::entities::{action, tag};
use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set, TransactionError, TransactionTrait};

use super::action_query::ActionQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewAction {
    pub name: String,
    pub description: Option<String>,
    pub user_id: uuid::Uuid,
}

pub struct ActionMutation;

impl ActionMutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewAction,
    ) -> Result<action::Model, TransactionError<DbErr>> {
        db.transaction::<_, action::Model, DbErr>(|txn| {
            Box::pin(async move {
                let now = Utc::now();
                let action_id = uuid::Uuid::new_v4();
                let created_action = action::ActiveModel {
                    id: Set(action_id),
                    user_id: Set(form_data.user_id),
                    name: Set(form_data.name.to_owned()),
                    description: Set(form_data.description.to_owned()),
                    archived: Set(false),
                    ordering: NotSet,
                    trackable: Set(true),
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    user_id: Set(form_data.user_id),
                    ambition_id: NotSet,
                    objective_id: NotSet,
                    action_id: Set(Some(action_id)),
                    created_at: Set(now.into()),
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
        description: Option<String>,
    ) -> Result<action::Model, DbErr> {
        let mut action: action::ActiveModel =
            ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
                .await?
                .into();
        action.name = Set(name);
        action.description = Set(description);
        action.updated_at = Set(Utc::now().into());
        action.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn archive(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<action::Model, DbErr> {
        let mut action: action::ActiveModel =
            ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
                .await?
                .into();
        action.archived = Set(true);
        action.updated_at = Set(Utc::now().into());
        action.update(db).await
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::DbErr;

    use crate::entities::tag;
    use crate::test_utils::{self, *};
    use crate::types::CustomDbErr;

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action_name = "create_with_tag".to_string();
        let action_description = "Create with Tag.".to_string();

        let form_data = NewAction {
            name: action_name.clone(),
            description: Some(action_description.clone()),
            user_id: user.id,
        };

        let returned_action = ActionMutation::create_with_tag(&db, form_data)
            .await
            .unwrap();
        assert_eq!(returned_action.name, action_name.clone());
        assert_eq!(
            returned_action.description,
            Some(action_description.clone())
        );
        assert_eq!(returned_action.archived, false);
        assert_eq!(returned_action.user_id, user.id);

        let created_action = action::Entity::find_by_id(returned_action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_action.name, action_name.clone());
        assert_eq!(created_action.description, Some(action_description.clone()));
        assert_eq!(created_action.archived, false);
        assert_eq!(created_action.user_id, user.id);
        assert_eq!(created_action.created_at, returned_action.created_at);
        assert_eq!(created_action.updated_at, returned_action.updated_at);

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
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let new_name = "action_after_update".to_string();
        let new_description = "Action after update.".to_string();

        let returned_action = ActionMutation::update(
            &db,
            action.id,
            user.id,
            new_name.clone(),
            Some(new_description.clone()),
        )
        .await?;
        assert_eq!(returned_action.id, action.id);
        assert_eq!(returned_action.name, new_name.clone());
        assert_eq!(returned_action.description, Some(new_description.clone()));
        assert_eq!(returned_action.archived, action.archived);
        assert_eq!(returned_action.user_id, user.id);
        assert_eq!(returned_action.created_at, action.created_at);
        assert!(returned_action.updated_at > action.updated_at);

        let updated_action = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_action.id, action.id);
        assert_eq!(updated_action.name, new_name.clone());
        assert_eq!(updated_action.description, Some(new_description.clone()));
        assert_eq!(updated_action.archived, action.archived);
        assert_eq!(updated_action.user_id, user.id);
        assert_eq!(updated_action.created_at, action.created_at);
        assert_eq!(updated_action.updated_at, returned_action.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let new_name = "action_after_update_unauthorized".to_string();

        let error =
            ActionMutation::update(&db, action.id, uuid::Uuid::new_v4(), new_name.clone(), None)
                .await
                .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let (action, tag) =  factory::action(user.id).insert_with_tag(&db).await?;

        ActionMutation::delete(&db, action.id, user.id).await?;

        let action_in_db = action::Entity::find_by_id(action.id).one(&db).await?;
        assert!(action_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let error = ActionMutation::delete(&db, action.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn archive() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        ActionMutation::archive(&db, action.id, user.id).await?;

        let action_in_db = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert!(action_in_db.archived);

        Ok(())
    }
}
