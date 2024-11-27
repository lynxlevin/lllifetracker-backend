use crate::entities::{memo, memos_tags};
use chrono::Utc;
use sea_orm::{
    entity::prelude::*, ActiveValue::NotSet, Condition, DeriveColumn, EnumIter, QuerySelect, Set,
    TransactionError, TransactionTrait,
};

use super::memo_query::MemoQuery;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewMemo {
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct UpdateMemo {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub text: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub tag_ids: Option<Vec<uuid::Uuid>>,
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

    pub async fn partial_update(
        db: &DbConn,
        form: UpdateMemo,
    ) -> Result<memo::Model, TransactionError<DbErr>> {
        let memo_result = MemoQuery::find_by_id_and_user_id(db, form.id, form.user_id).await;
        db.transaction::<_, memo::Model, DbErr>(|txn| {
            Box::pin(async move {
                let mut memo: memo::ActiveModel = memo_result?.into();
                if let Some(title) = form.title {
                    memo.title = Set(title);
                }
                if let Some(text) = form.text {
                    memo.text = Set(text);
                }
                if let Some(date) = form.date {
                    memo.date = Set(date);
                }
                if let Some(tag_ids) = form.tag_ids {
                    let linked_tag_ids = memos_tags::Entity::find()
                        .column_as(memos_tags::Column::TagId, QueryAs::TagId)
                        .filter(memos_tags::Column::MemoId.eq(form.id))
                        .into_values::<uuid::Uuid, QueryAs>()
                        .all(txn)
                        .await?;

                    let tag_links_to_create: Vec<memos_tags::ActiveModel> = tag_ids
                        .clone()
                        .into_iter()
                        .filter(|id| !linked_tag_ids.contains(id))
                        .map(|tag_id| memos_tags::ActiveModel {
                            memo_id: Set(form.id),
                            tag_id: Set(tag_id),
                        })
                        .collect();
                    memos_tags::Entity::insert_many(tag_links_to_create)
                        .on_empty_do_nothing()
                        .exec(txn)
                        .await?;

                    let ids_to_delete: Vec<uuid::Uuid> = linked_tag_ids
                        .into_iter()
                        .filter(|linked_tag_id| !tag_ids.contains(linked_tag_id))
                        .collect();
                    if ids_to_delete.len() > 0 {
                        memos_tags::Entity::delete_many()
                            .filter(memos_tags::Column::MemoId.eq(form.id))
                            .filter(memos_tags::Column::TagId.is_in(ids_to_delete))
                            .exec(txn)
                            .await?;
                    }
                }
                memo.updated_at = Set(Utc::now().into());
                memo.update(txn).await
            })
        })
        .await
    }

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

    use chrono::Datelike;
    use sea_orm::{DbErr, EntityOrSelect};

    use crate::entities::memos_tags;
    use crate::test_utils;
    use crate::types::CustomDbErr;

    use super::*;

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
            .column_as(memos_tags::Column::TagId, QueryAs::TagId)
            .filter(memos_tags::Column::MemoId.eq(returned_memo.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 2);
        assert!(linked_tag_ids.contains(&tag_0.id));
        assert!(linked_tag_ids.contains(&tag_1.id));

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_title() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo =
            test_utils::seed::create_memo(&db, "Memo without tags".to_string(), user.id).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: Some("Updated Memo".to_string()),
            text: None,
            date: None,
            tag_ids: None,
            user_id: user.id,
        };

        let returned_memo = MemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_memo.id, memo.id);
        assert_eq!(returned_memo.title, form.title.clone().unwrap());
        assert_eq!(returned_memo.text, memo.text);
        assert_eq!(returned_memo.date, memo.date);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id)
            .filter(memo::Column::Title.eq(form.title.clone().unwrap()))
            .filter(memo::Column::Text.eq(memo.text))
            .filter(memo::Column::Date.eq(memo.date))
            .filter(memo::Column::UserId.eq(user.id))
            .filter(memo::Column::CreatedAt.eq(memo.created_at))
            .filter(memo::Column::UpdatedAt.eq(returned_memo.updated_at))
            .one(&db)
            .await?;
        assert!(updated_memo.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_text() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo =
            test_utils::seed::create_memo(&db, "Memo without tags".to_string(), user.id).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: Some("Updated memo content.".to_string()),
            date: None,
            tag_ids: None,
            user_id: user.id,
        };

        let returned_memo = MemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_memo.id, memo.id);
        assert_eq!(returned_memo.title, memo.title);
        assert_eq!(returned_memo.text, form.text.clone().unwrap());
        assert_eq!(returned_memo.date, memo.date);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id)
            .filter(memo::Column::Title.eq(memo.title))
            .filter(memo::Column::Text.eq(form.text.clone().unwrap()))
            .filter(memo::Column::Date.eq(memo.date))
            .filter(memo::Column::UserId.eq(user.id))
            .filter(memo::Column::CreatedAt.eq(memo.created_at))
            .filter(memo::Column::UpdatedAt.eq(returned_memo.updated_at))
            .one(&db)
            .await?;
        assert!(updated_memo.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_date() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo =
            test_utils::seed::create_memo(&db, "Memo without tags".to_string(), user.id).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: Some(chrono::Utc::now().with_year(1900).unwrap().date_naive()),
            tag_ids: None,
            user_id: user.id,
        };

        let returned_memo = MemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_memo.id, memo.id);
        assert_eq!(returned_memo.title, memo.title);
        assert_eq!(returned_memo.text, memo.text);
        assert_eq!(returned_memo.date, form.date.unwrap());
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id)
            .filter(memo::Column::Title.eq(memo.title))
            .filter(memo::Column::Text.eq(memo.text))
            .filter(memo::Column::Date.eq(form.date.unwrap()))
            .filter(memo::Column::UserId.eq(user.id))
            .filter(memo::Column::CreatedAt.eq(memo.created_at))
            .filter(memo::Column::UpdatedAt.eq(returned_memo.updated_at))
            .one(&db)
            .await?;
        assert!(updated_memo.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_add_tags() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo =
            test_utils::seed::create_memo(&db, "Memo without tags".to_string(), user.id).await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: None,
            tag_ids: Some(vec![ambition_tag.id]),
            user_id: user.id,
        };

        let returned_memo = MemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_memo.id, memo.id);
        assert_eq!(returned_memo.title, memo.title);
        assert_eq!(returned_memo.text, memo.text);
        assert_eq!(returned_memo.date, memo.date);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id)
            .filter(memo::Column::Title.eq(memo.title))
            .filter(memo::Column::Text.eq(memo.text))
            .filter(memo::Column::Date.eq(memo.date))
            .filter(memo::Column::UserId.eq(user.id))
            .filter(memo::Column::CreatedAt.eq(memo.created_at))
            .filter(memo::Column::UpdatedAt.eq(returned_memo.updated_at))
            .one(&db)
            .await?;
        assert!(updated_memo.is_some());

        let linked_tag_ids: Vec<uuid::Uuid> = memos_tags::Entity::find()
            .column_as(memos_tags::Column::TagId, QueryAs::TagId)
            .filter(memos_tags::Column::MemoId.eq(returned_memo.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 1);
        assert!(linked_tag_ids.contains(&ambition_tag.id));

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_remove_tags() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo =
            test_utils::seed::create_memo(&db, "Memo without tags".to_string(), user.id).await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        memos_tags::ActiveModel {
            memo_id: Set(memo.id),
            tag_id: Set(ambition_tag.id),
        }
        .insert(&db)
        .await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: None,
            tag_ids: Some(vec![]),
            user_id: user.id,
        };

        let returned_memo = MemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_memo.id, memo.id);
        assert_eq!(returned_memo.title, memo.title);
        assert_eq!(returned_memo.text, memo.text);
        assert_eq!(returned_memo.date, memo.date);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id)
            .filter(memo::Column::Title.eq(memo.title))
            .filter(memo::Column::Text.eq(memo.text))
            .filter(memo::Column::Date.eq(memo.date))
            .filter(memo::Column::UserId.eq(user.id))
            .filter(memo::Column::CreatedAt.eq(memo.created_at))
            .filter(memo::Column::UpdatedAt.eq(returned_memo.updated_at))
            .one(&db)
            .await?;
        assert!(updated_memo.is_some());

        let linked_tag_ids: Vec<uuid::Uuid> = memos_tags::Entity::find()
            .column_as(memos_tags::Column::TagId, QueryAs::TagId)
            .filter(memos_tags::Column::MemoId.eq(returned_memo.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 0);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo =
            test_utils::seed::create_memo(&db, "Memo without tags".to_string(), user.id).await?;
        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: None,
            tag_ids: None,
            user_id: uuid::Uuid::new_v4(),
        };

        let error = MemoMutation::partial_update(&db, form).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            TransactionError::Transaction(DbErr::Custom(CustomDbErr::NotFound.to_string()))
                .to_string(),
        );

        Ok(())
    }

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
