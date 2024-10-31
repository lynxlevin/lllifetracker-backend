use crate::entities::objective;
use crate::types::{CustomDbErr, ObjectiveVisible};
use sea_orm::entity::prelude::*;
use sea_orm::QueryOrder;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewObjective {
    pub name: String,
    pub user_id: uuid::Uuid,
}

pub struct ObjectiveQuery;

impl ObjectiveQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ObjectiveVisible>, DbErr> {
        objective::Entity::find()
            .filter(objective::Column::UserId.eq(user_id))
            .order_by_asc(objective::Column::CreatedAt)
            .into_partial_model::<ObjectiveVisible>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        objective_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<objective::Model, DbErr> {
        objective::Entity::find_by_id(objective_id)
            .filter(objective::Column::UserId.eq(user_id))
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
        let (objective_1, _) =
            test_utils::seed::create_objective_and_tag(&db, "objective_1".to_string(), user.id)
                .await?;
        let (objective_2, _) =
            test_utils::seed::create_objective_and_tag(&db, "objective_2".to_string(), user.id)
                .await?;

        let res = ObjectiveQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = vec![
            ObjectiveVisible {
                id: objective_1.id,
                name: objective_1.name,
                created_at: objective_1.created_at,
                updated_at: objective_1.updated_at,
            },
            ObjectiveVisible {
                id: objective_2.id,
                name: objective_2.name,
                created_at: objective_2.created_at,
                updated_at: objective_2.updated_at,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);

        Ok(())
    }
}
