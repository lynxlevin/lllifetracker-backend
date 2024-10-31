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

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (action_1, _) =
            test_utils::seed::create_action_and_tag(&db, "action_1".to_string(), user.id).await?;
        let (action_2, _) =
            test_utils::seed::create_action_and_tag(&db, "action_2".to_string(), user.id).await?;

        let res = ActionQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = vec![
            ActionVisible {
                id: action_1.id,
                name: action_1.name,
                created_at: action_1.created_at,
                updated_at: action_1.updated_at,
            },
            ActionVisible {
                id: action_2.id,
                name: action_2.name,
                created_at: action_2.created_at,
                updated_at: action_2.updated_at,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);

        Ok(())
    }
}
