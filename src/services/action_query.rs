use crate::entities::action;
use crate::types::{ActionVisible, CustomDbErr};
use sea_orm::{entity::prelude::*, QueryOrder};

pub struct ActionQuery;

impl ActionQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ActionVisible>, DbErr> {
        action::Entity::find()
            .filter(action::Column::UserId.eq(user_id))
            .order_by_asc(action::Column::CreatedAt)
            .into_partial_model::<ActionVisible>()
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
