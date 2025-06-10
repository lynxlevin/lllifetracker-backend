use ::types::{AmbitionVisible, CustomDbErr};
use entities::ambition;
use sea_orm::{
    sea_query::NullOrdering::Last, ColumnTrait, DbConn, DbErr, EntityTrait, Order::Asc,
    QueryFilter, QueryOrder,
};

pub struct AmbitionQuery;

impl AmbitionQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
        show_archived_only: bool,
    ) -> Result<Vec<AmbitionVisible>, DbErr> {
        ambition::Entity::find()
            .filter(ambition::Column::UserId.eq(user_id))
            .filter(ambition::Column::Archived.eq(show_archived_only))
            .order_by_with_nulls(ambition::Column::Ordering, Asc, Last)
            .order_by_asc(ambition::Column::CreatedAt)
            .into_partial_model::<AmbitionVisible>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<ambition::Model, DbErr> {
        ambition::Entity::find_by_id(ambition_id)
            .filter(ambition::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use test_utils;

    use common::factory::{self, *};
    use sea_orm::ActiveModelTrait;

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id)
            .name("ambition_0".to_string())
            .description(Some("desc_0".to_string()))
            .insert(&db)
            .await?;
        let ambition_1 = factory::ambition(user.id)
            .name("ambition_1".to_string())
            .ordering(Some(2))
            .insert(&db)
            .await?;
        let ambition_2 = factory::ambition(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = AmbitionQuery::find_all_by_user_id(&db, user.id, false).await?;

        let expected = [
            AmbitionVisible {
                id: ambition_2.id,
                name: ambition_2.name,
                description: ambition_2.description,
                created_at: ambition_2.created_at,
                updated_at: ambition_2.updated_at,
            },
            AmbitionVisible {
                id: ambition_1.id,
                name: ambition_1.name,
                description: ambition_1.description,
                created_at: ambition_1.created_at,
                updated_at: ambition_1.updated_at,
            },
            AmbitionVisible {
                id: ambition_0.id,
                name: ambition_0.name,
                description: ambition_0.description,
                created_at: ambition_0.created_at,
                updated_at: ambition_0.updated_at,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_by_user_id_show_archived_only() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let _ambition = factory::ambition(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = AmbitionQuery::find_all_by_user_id(&db, user.id, true).await?;

        let expected = [AmbitionVisible {
            id: archived_ambition.id,
            name: archived_ambition.name,
            description: archived_ambition.description,
            created_at: archived_ambition.created_at,
            updated_at: archived_ambition.updated_at,
        }];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }
}
