use entities::mindset;
use sea_orm::{
    sea_query::NullOrdering::Last, ColumnTrait, DbConn, DbErr, EntityTrait, Order::Asc,
    QueryFilter, QueryOrder,
};
use types::{CustomDbErr, MindsetVisible};

pub struct MindsetQuery;

impl MindsetQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
        show_archived_only: bool,
    ) -> Result<Vec<MindsetVisible>, DbErr> {
        mindset::Entity::find()
            .filter(mindset::Column::UserId.eq(user_id))
            .filter(mindset::Column::Archived.eq(show_archived_only))
            .order_by_with_nulls(mindset::Column::Ordering, Asc, Last)
            .order_by_asc(mindset::Column::CreatedAt)
            .into_partial_model::<MindsetVisible>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        mindset_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<mindset::Model, DbErr> {
        mindset::Entity::find_by_id(mindset_id)
            .filter(mindset::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use common::factory::{self, *};
    use test_utils;

    use sea_orm::ActiveModelTrait;

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let mindset_0 = factory::mindset(user.id)
            .name("mindset_0".to_string())
            .description(Some("desc_0".to_string()))
            .insert(&db)
            .await?;
        let mindset_1 = factory::mindset(user.id)
            .name("mindset_1".to_string())
            .ordering(Some(2))
            .insert(&db)
            .await?;
        let mindset_2 = factory::mindset(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        let _archived_mindset = factory::mindset(user.id).archived(true).insert(&db).await?;

        let res = MindsetQuery::find_all_by_user_id(&db, user.id, false).await?;

        let expected = [
            MindsetVisible {
                id: mindset_2.id,
                name: mindset_2.name,
                description: mindset_2.description,
                created_at: mindset_2.created_at,
                updated_at: mindset_2.updated_at,
            },
            MindsetVisible {
                id: mindset_1.id,
                name: mindset_1.name,
                description: mindset_1.description,
                created_at: mindset_1.created_at,
                updated_at: mindset_1.updated_at,
            },
            MindsetVisible {
                id: mindset_0.id,
                name: mindset_0.name,
                description: mindset_0.description,
                created_at: mindset_0.created_at,
                updated_at: mindset_0.updated_at,
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
        let _mindset = factory::mindset(user.id).insert(&db).await?;
        let archived_mindset = factory::mindset(user.id).archived(true).insert(&db).await?;

        let res = MindsetQuery::find_all_by_user_id(&db, user.id, true).await?;

        let expected = [MindsetVisible {
            id: archived_mindset.id,
            name: archived_mindset.name,
            description: archived_mindset.description,
            created_at: archived_mindset.created_at,
            updated_at: archived_mindset.updated_at,
        }];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }
}
