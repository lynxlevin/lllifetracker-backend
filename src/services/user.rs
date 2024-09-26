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
            email: Set(form_data.email.to_owned()),
            password: Set(form_data.password.to_owned()),
            first_name: Set(form_data.first_name.to_owned()),
            last_name: Set(form_data.last_name.to_owned()),
            is_active: Set(form_data.is_active.to_owned()),
            ..Default::default()
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
