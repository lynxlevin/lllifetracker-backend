use crate::entities::sea_orm_active_enums::TimezoneEnum;
use crate::entities::user;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::Set;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
}

pub struct Mutation;

impl Mutation {
    pub async fn create_user(db: &DbConn, form_data: NewUser) -> Result<user::Model, DbErr> {
        user::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            email: Set(form_data.email),
            password: Set(form_data.password),
            first_name: Set(form_data.first_name),
            last_name: Set(form_data.last_name),
            is_active: Set(form_data.is_active),
            timezone: Set(TimezoneEnum::AsiaTokyo),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        }
        .insert(db)
        .await
    }

    pub async fn activate_user_by_id(db: &DbConn, id: uuid::Uuid) -> Result<user::Model, DbErr> {
        let mut user: user::ActiveModel = match Query::find_by_id(db, id).await {
            Ok(user) => user.unwrap().into(),
            Err(e) => {
                return Err(e);
            }
        };
        user.is_active = Set(true);
        user.updated_at = Set(Utc::now().into());
        user.update(db).await
    }

    pub async fn update_user_password(
        db: &DbConn,
        id: uuid::Uuid,
        password: String,
    ) -> Result<user::Model, DbErr> {
        match Query::find_by_id(db, id).await {
            Ok(_user) => {
                let mut user: user::ActiveModel = _user.unwrap().into();
                user.password = Set(password);
                user.updated_at = Set(Utc::now().into());
                user.update(db).await
            }
            Err(e) => Err(e),
        }
    }
}

pub struct Query;

impl Query {
    pub async fn find_by_id(db: &DbConn, id: uuid::Uuid) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find_by_id(id).one(db).await
    }

    pub async fn find_active_by_email(
        db: &DbConn,
        email: String,
    ) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .filter(user::Column::IsActive.eq(true))
            .one(db)
            .await
    }

    pub async fn find_inactive_by_email(
        db: &DbConn,
        email: String,
    ) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .filter(user::Column::IsActive.eq(false))
            .one(db)
            .await
    }
}

#[cfg(test)]
mod mutation_tests {
    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn create_user() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let form_data = NewUser {
            email: "test@test.com".to_string(),
            password: "password".to_string(),
            first_name: "Lynx".to_string(),
            last_name: "Levin".to_string(),
            is_active: false,
        };

        let returned_user = Mutation::create_user(&db, form_data.clone()).await?;
        assert_eq!(returned_user.email, form_data.email);
        assert_eq!(returned_user.password, form_data.password);
        assert_eq!(returned_user.first_name, form_data.first_name);
        assert_eq!(returned_user.last_name, form_data.last_name);
        assert_eq!(returned_user.is_active, form_data.is_active);

        let created_user = user::Entity::find_by_id(returned_user.id)
            .filter(user::Column::Email.eq(form_data.email))
            .filter(user::Column::Password.eq(form_data.password))
            .filter(user::Column::FirstName.eq(form_data.first_name))
            .filter(user::Column::LastName.eq(form_data.last_name))
            .filter(user::Column::IsActive.eq(form_data.is_active))
            .filter(user::Column::Timezone.eq("Asia/Tokyo".to_string()))
            .filter(user::Column::Timezone.eq(TimezoneEnum::AsiaTokyo))
            .one(&db)
            .await?;
        assert!(created_user.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn activate_user_by_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_inactive_user(&db).await?;

        let returned_user = Mutation::activate_user_by_id(&db, user.id).await?;
        assert_eq!(returned_user.id, user.id);
        assert_eq!(returned_user.is_active, true);
        assert!(returned_user.updated_at > user.updated_at);

        let activated_user = user::Entity::find_by_id(user.id)
            .filter(user::Column::Email.eq(user.email))
            .filter(user::Column::Password.eq(user.password))
            .filter(user::Column::FirstName.eq(user.first_name))
            .filter(user::Column::LastName.eq(user.last_name))
            .filter(user::Column::Timezone.eq(user.timezone))
            .filter(user::Column::IsActive.eq(true))
            .one(&db)
            .await?;
        assert!(activated_user.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update_user_password() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let new_password = "updated_password".to_string();

        let returned_user =
            Mutation::update_user_password(&db, user.id, new_password.clone()).await?;
        assert_eq!(returned_user.id, user.id);
        assert_eq!(returned_user.password, new_password);

        let updated_user = user::Entity::find_by_id(user.id)
            .filter(user::Column::Email.eq(user.email))
            .filter(user::Column::Password.eq(returned_user.password))
            .filter(user::Column::FirstName.eq(user.first_name))
            .filter(user::Column::LastName.eq(user.last_name))
            .filter(user::Column::Timezone.eq(user.timezone))
            .filter(user::Column::IsActive.eq(user.is_active))
            .one(&db)
            .await?;
        assert!(updated_user.is_some());

        Ok(())
    }
}
