use crate::entities::{action, tag};
use crate::types::CustomDbErr;
use chrono::Utc;
use sea_orm::entity::prelude::*;
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
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    user_id: Set(form_data.user_id),
                    action_id: Set(Some(action_id)),
                    ..Default::default()
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

// MYMEMO: Would also like to try MockDatabase, but to do that, need to change the entire structure.
// https://github.com/SeaQL/sea-orm/issues/830
#[cfg(test)]
mod tests {
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{DbConn, DbErr};

    use crate::{
        entities::{tag, user},
        startup::get_database_connection,
    };

    use super::*;

    #[actix_web::test]
    async fn main() -> Result<(), DbErr> {
        dotenvy::from_filename(".env.test").unwrap();
        // NOTE: Ideally, Sqlite should be used instead of Postgres but cannot,
        // because programmatic migration for Sqlite using Migrator is not supported
        // nor migration from entities do not set default constraints.
        let db = get_database_connection().await;
        Migrator::up(&db, None).await.unwrap();

        flush(&db).await?;
        test_create_with_tag(&db).await?;

        Ok(())
    }

    async fn flush(db: &DbConn) -> Result<(), DbErr> {
        tag::Entity::delete_many().exec(db).await?;
        action::Entity::delete_many().exec(db).await?;
        Ok(())
    }

    async fn test_create_with_tag(db: &DbConn) -> Result<(), DbErr> {
        let user = user::Entity::find()
            .filter(user::Column::Email.eq("test@test.com".to_string()))
            .one(db)
            .await?
            .unwrap();

        let form_data = NewAction {
            name: "Test action".to_string(),
            user_id: user.id,
        };

        let created_action = Mutation::create_with_tag(db, form_data).await.unwrap();
        assert_eq!(created_action.name, "Test action".to_string());
        assert_eq!(created_action.user_id, user.id);

        let action_from_db = action::Entity::find_by_id(created_action.id)
            .filter(action::Column::Name.eq("Test action".to_string()))
            .filter(action::Column::UserId.eq(user.id))
            .one(db)
            .await?;
        assert_ne!(action_from_db, None);

        let tag_from_db = tag::Entity::find()
            .filter(tag::Column::ActionId.eq(created_action.id))
            .filter(tag::Column::UserId.eq(user.id))
            .one(db)
            .await?;
        assert_ne!(tag_from_db, None);

        Ok(())
    }
}
