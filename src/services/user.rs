use chrono::Utc;
use entities::{sea_orm_active_enums::TimezoneEnum, user};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set};

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
        let now = Utc::now();
        user::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            email: Set(form_data.email),
            password: Set(form_data.password),
            first_name: Set(form_data.first_name),
            last_name: Set(form_data.last_name),
            is_active: Set(form_data.is_active),
            timezone: Set(TimezoneEnum::AsiaTokyo),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
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
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn create_user() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let form_data = NewUser {
            email: format!("{}@test.com", uuid::Uuid::now_v7().to_string()),
            password: "password".to_string(),
            first_name: "Lynx".to_string(),
            last_name: "Levin".to_string(),
            is_active: false,
        };

        let res = Mutation::create_user(&db, form_data.clone()).await?;
        assert_eq!(res.email, form_data.email);
        assert_eq!(res.password, form_data.password);
        assert_eq!(res.first_name, form_data.first_name);
        assert_eq!(res.last_name, form_data.last_name);
        assert_eq!(res.is_active, form_data.is_active);
        assert_eq!(res.timezone, TimezoneEnum::AsiaTokyo);

        let user_in_db = user::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(user_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn activate_user_by_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().is_active(false).insert(&db).await?;

        let res = Mutation::activate_user_by_id(&db, user.id).await?;
        assert_eq!(res.id, user.id);
        assert_eq!(res.email, user.email);
        assert_eq!(res.password, user.password);
        assert_eq!(res.first_name, user.first_name);
        assert_eq!(res.last_name, user.last_name);
        assert_eq!(res.is_active, true);
        assert_eq!(res.timezone, TimezoneEnum::AsiaTokyo);
        assert_eq!(res.created_at, user.created_at);
        assert!(res.updated_at > user.updated_at);

        let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
        assert_eq!(user_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn update_user_password() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let new_password = "updated_password".to_string();

        let res = Mutation::update_user_password(&db, user.id, new_password.clone()).await?;
        assert_eq!(res.id, user.id);
        assert_eq!(res.email, user.email);
        assert_eq!(res.password, new_password);
        assert_eq!(res.first_name, user.first_name);
        assert_eq!(res.last_name, user.last_name);
        assert_eq!(res.is_active, true);
        assert_eq!(res.timezone, TimezoneEnum::AsiaTokyo);
        assert_eq!(res.created_at, user.created_at);
        assert!(res.updated_at > user.updated_at);

        let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
        assert_eq!(user_in_db, res);

        Ok(())
    }
}
