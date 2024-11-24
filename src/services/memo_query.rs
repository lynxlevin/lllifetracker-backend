use crate::entities::memo;
use crate::types::{CustomDbErr, MemoVisible};
use sea_orm::entity::prelude::*;
use sea_orm::QueryOrder;

pub struct MemoQuery;

impl MemoQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<MemoVisible>, DbErr> {
        memo::Entity::find()
            .filter(memo::Column::UserId.eq(user_id))
            .order_by_desc(memo::Column::CreatedAt)
            .into_partial_model::<MemoVisible>()
            .all(db)
            .await
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
        let memo_0 = test_utils::seed::create_memo(&db, "memo_0".to_string(), user.id).await?;
        let memo_1 = test_utils::seed::create_memo(&db, "memo_1".to_string(), user.id).await?;

        let res = MemoQuery::find_all_by_user_id(&db, user.id).await?;

        // MYMEMO: change to return MemoVisibleWithTags
        let expected = vec![
            MemoVisible {
                id: memo_1.id,
                title: memo_1.title,
                text: memo_1.text,
                date: memo_1.date,
                created_at: memo_1.created_at,
                updated_at: memo_1.updated_at,
            },
            MemoVisible {
                id: memo_0.id,
                title: memo_0.title,
                text: memo_0.text,
                date: memo_0.date,
                created_at: memo_0.created_at,
                updated_at: memo_0.updated_at,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);

        Ok(())
    }
}
