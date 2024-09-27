use crate::entities::action;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryOrder, Set};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewAction {
    pub name: String,
    pub user_id: uuid::Uuid,
}

pub struct Mutation;

impl Mutation {
    pub async fn create(db: &DbConn, form_data: NewAction) -> Result<action::Model, DbErr> {
        action::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set(form_data.user_id),
            name: Set(form_data.name.to_owned()),
            ..Default::default()
        }
        .insert(db)
        .await
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
    ) -> Result<Option<action::Model>, DbErr> {
        action::Entity::find_by_id(action_id)
            .filter(action::Column::UserId.eq(user_id))
            .one(db)
            .await
    }
}
