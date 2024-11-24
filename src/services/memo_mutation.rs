use crate::entities::{memo, memos_tags, tag};
use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set, TransactionError, TransactionTrait};

use super::memo_query::MemoQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewMemo {
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

pub struct MemoMutation;

impl MemoMutation {
    pub async fn create(
        db: &DbConn,
        form_data: NewMemo,
    ) -> Result<memo::Model, TransactionError<DbErr>> {
        db.transaction::<_, memo::Model, DbErr>(|txn| {
            Box::pin(async move {
                let now = Utc::now();
                let memo_id = uuid::Uuid::new_v4();
                let created_memo = memo::ActiveModel {
                    id: Set(memo_id),
                    user_id: Set(form_data.user_id),
                    title: Set(form_data.title.to_owned()),
                    text: Set(form_data.text.to_owned()),
                    date: Set(form_data.date),
                    archived: Set(false),
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                }
                .insert(txn)
                .await?;

                for tag_id in form_data.tag_ids {
                    memos_tags::ActiveModel {
                        memo_id: Set(created_memo.id),
                        tag_id: Set(tag_id),
                    }
                    .insert(txn)
                    .await?;
                }

                Ok(created_memo)
            })
        })
        .await
    }

    // pub async fn update(
    //     db: &DbConn,
    //     action_id: uuid::Uuid,
    //     user_id: uuid::Uuid,
    //     name: String,
    // ) -> Result<action::Model, DbErr> {
    //     let mut action: action::ActiveModel =
    //         MemoQuery::find_by_id_and_user_id(db, action_id, user_id)
    //             .await?
    //             .into();
    //     action.name = Set(name);
    //     action.updated_at = Set(Utc::now().into());
    //     action.update(db).await
    // }

    // pub async fn delete(
    //     db: &DbConn,
    //     action_id: uuid::Uuid,
    //     user_id: uuid::Uuid,
    // ) -> Result<(), DbErr> {
    //     MemoQuery::find_by_id_and_user_id(db, action_id, user_id)
    //         .await?
    //         .delete(db)
    //         .await?;
    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbErr, DeriveColumn, EntityOrSelect, EnumIter, QuerySelect};

    use crate::entities::memos_tags;
    use crate::test_utils;
    use crate::types::{CustomDbErr, TagType};

    use super::*;

    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    enum QueryAs {
        MemoId,
    }

    #[actix_web::test]
    async fn create() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (_, tag_0) =
            test_utils::seed::create_action_and_tag(&db, "action_0".to_string(), user.id).await?;
        let (_, tag_1) =
            test_utils::seed::create_action_and_tag(&db, "action_1".to_string(), user.id).await?;
        let memo_title = "New Memo".to_string();
        let memo_text = "This is a new memo for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();

        let form_data = NewMemo {
            title: memo_title.clone(),
            text: memo_text.clone(),
            date: today,
            tag_ids: vec![tag_0.id, tag_1.id],
            user_id: user.id,
        };

        let returned_memo = MemoMutation::create(&db, form_data).await.unwrap();
        assert_eq!(returned_memo.title, memo_title.clone());
        assert_eq!(returned_memo.text, memo_text.clone());
        assert_eq!(returned_memo.date, today);
        assert_eq!(returned_memo.user_id, user.id);

        let created_memo = memo::Entity::find_by_id(returned_memo.id)
            .filter(memo::Column::Title.eq(memo_title.clone()))
            .filter(memo::Column::Text.eq(memo_text.clone()))
            .filter(memo::Column::Date.eq(today))
            .filter(memo::Column::UserId.eq(user.id))
            .filter(memo::Column::CreatedAt.eq(returned_memo.created_at))
            .filter(memo::Column::UpdatedAt.eq(returned_memo.updated_at))
            .one(&db)
            .await?;
        assert!(created_memo.is_some());

        let linked_tag_ids: Vec<uuid::Uuid> = memos_tags::Entity::find()
            .column_as(memos_tags::Column::TagId, QueryAs::MemoId)
            .filter(memos_tags::Column::MemoId.eq(returned_memo.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 2);
        assert!(linked_tag_ids.contains(&tag_0.id));
        assert!(linked_tag_ids.contains(&tag_1.id));

        Ok(())
    }

    // #[actix_web::test]
    // async fn update() -> Result<(), DbErr> {
    //     let db = test_utils::init_db().await?;
    //     let user = test_utils::seed::create_active_user(&db).await?;
    //     let (action, _) = test_utils::seed::create_action_and_tag(
    //         &db,
    //         "action_before_update".to_string(),
    //         user.id,
    //     )
    //     .await?;
    //     let new_name = "action_after_update".to_string();

    //     let returned_action =
    //         MemoMutation::update(&db, action.id, user.id, new_name.clone()).await?;
    //     assert_eq!(returned_action.id, action.id);
    //     assert_eq!(returned_action.name, new_name.clone());
    //     assert_eq!(returned_action.user_id, user.id);
    //     assert_eq!(returned_action.created_at, action.created_at);
    //     assert!(returned_action.updated_at > action.updated_at);

    //     let updated_action = action::Entity::find_by_id(action.id)
    //         .filter(action::Column::Name.eq(new_name))
    //         .filter(action::Column::UserId.eq(user.id))
    //         .filter(action::Column::CreatedAt.eq(action.created_at))
    //         .filter(action::Column::UpdatedAt.eq(returned_action.updated_at))
    //         .one(&db)
    //         .await?;
    //     assert!(updated_action.is_some());

    //     Ok(())
    // }

    // #[actix_web::test]
    // async fn update_unauthorized() -> Result<(), DbErr> {
    //     let db = test_utils::init_db().await?;
    //     let user = test_utils::seed::create_active_user(&db).await?;
    //     let (action, _) = test_utils::seed::create_action_and_tag(
    //         &db,
    //         "action_before_update_unauthorized".to_string(),
    //         user.id,
    //     )
    //     .await?;
    //     let new_name = "action_after_update_unauthorized".to_string();

    //     let error = MemoMutation::update(&db, action.id, uuid::Uuid::new_v4(), new_name.clone())
    //         .await
    //         .unwrap_err();
    //     assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

    //     Ok(())
    // }

    // #[actix_web::test]
    // async fn delete() -> Result<(), DbErr> {
    //     let db = test_utils::init_db().await?;
    //     let user = test_utils::seed::create_active_user(&db).await?;
    //     let (action, tag) =
    //         test_utils::seed::create_action_and_tag(&db, "action_for_delete".to_string(), user.id)
    //             .await?;

    //     MemoMutation::delete(&db, action.id, user.id).await?;

    //     let action_in_db = action::Entity::find_by_id(action.id).one(&db).await?;
    //     assert!(action_in_db.is_none());

    //     let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
    //     assert!(tag_in_db.is_none());

    //     Ok(())
    // }

    // #[actix_web::test]
    // async fn delete_unauthorized() -> Result<(), DbErr> {
    //     let db = test_utils::init_db().await?;
    //     let user = test_utils::seed::create_active_user(&db).await?;
    //     let (action, _) = test_utils::seed::create_action_and_tag(
    //         &db,
    //         "action_for_delete_unauthorized".to_string(),
    //         user.id,
    //     )
    //     .await?;

    //     let error = MemoMutation::delete(&db, action.id, uuid::Uuid::new_v4())
    //         .await
    //         .unwrap_err();
    //     assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

    //     Ok(())
    // }
}
