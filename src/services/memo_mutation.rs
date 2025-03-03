use chrono::Utc;
use entities::{memo, memos_tags};
use sea_orm::{
    entity::prelude::*, DeriveColumn, EnumIter, QuerySelect, Set, TransactionError,
    TransactionTrait,
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
    pub favorite: Option<bool>,
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
                    favorite: Set(false),
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
                if let Some(favorite) = form.favorite {
                    memo.favorite = Set(favorite);
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

    pub async fn delete(
        db: &DbConn,
        memo_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        MemoQuery::find_by_id_and_user_id(db, memo_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use sea_orm::DbErr;

    use ::types::CustomDbErr;
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn create() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let (_, tag_0) = factory::action(user.id)
            .name("action_0".to_string())
            .insert_with_tag(&db)
            .await?;
        let (_, tag_1) = factory::action(user.id)
            .name("action_1".to_string())
            .insert_with_tag(&db)
            .await?;

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
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_memo.title, memo_title.clone());
        assert_eq!(created_memo.text, memo_text.clone());
        assert_eq!(created_memo.date, today);
        assert_eq!(created_memo.user_id, user.id);
        assert_eq!(created_memo.created_at, returned_memo.created_at);
        assert_eq!(created_memo.updated_at, returned_memo.updated_at);

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
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: Some("Updated Memo".to_string()),
            text: None,
            date: None,
            favorite: None,
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
        assert_eq!(returned_memo.favorite, memo.favorite);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id).one(&db).await?.unwrap();
        assert_eq!(updated_memo.title, form.title.clone().unwrap());
        assert_eq!(updated_memo.text, memo.text);
        assert_eq!(updated_memo.date, memo.date);
        assert_eq!(updated_memo.favorite, memo.favorite);
        assert_eq!(updated_memo.user_id, user.id);
        assert_eq!(updated_memo.created_at, memo.created_at);
        assert_eq!(updated_memo.updated_at, returned_memo.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_text() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: Some("Updated memo content.".to_string()),
            date: None,
            favorite: None,
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
        assert_eq!(returned_memo.favorite, memo.favorite);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id).one(&db).await?.unwrap();
        assert_eq!(updated_memo.title, memo.title);
        assert_eq!(updated_memo.text, form.text.clone().unwrap());
        assert_eq!(updated_memo.date, memo.date);
        assert_eq!(updated_memo.favorite, memo.favorite);
        assert_eq!(updated_memo.user_id, user.id);
        assert_eq!(updated_memo.created_at, memo.created_at);
        assert_eq!(updated_memo.updated_at, returned_memo.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_date() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: Some(chrono::Utc::now().with_year(1900).unwrap().date_naive()),
            favorite: None,
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
        assert_eq!(returned_memo.favorite, memo.favorite);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id).one(&db).await?.unwrap();
        assert_eq!(updated_memo.title, memo.title);
        assert_eq!(updated_memo.text, memo.text);
        assert_eq!(updated_memo.date, form.date.clone().unwrap());
        assert_eq!(updated_memo.favorite, memo.favorite);
        assert_eq!(updated_memo.user_id, user.id);
        assert_eq!(updated_memo.created_at, memo.created_at);
        assert_eq!(updated_memo.updated_at, returned_memo.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_favorite() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: None,
            favorite: Some(true),
            tag_ids: None,
            user_id: user.id,
        };

        let returned_memo = MemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_memo.id, memo.id);
        assert_eq!(returned_memo.title, memo.title);
        assert_eq!(returned_memo.text, memo.text);
        assert_eq!(returned_memo.date, memo.date);
        assert_eq!(returned_memo.favorite, form.favorite.unwrap());
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id).one(&db).await?.unwrap();
        assert_eq!(updated_memo.title, memo.title);
        assert_eq!(updated_memo.text, memo.text);
        assert_eq!(updated_memo.date, memo.date);
        assert_eq!(updated_memo.favorite, form.favorite.unwrap());
        assert_eq!(updated_memo.user_id, user.id);
        assert_eq!(updated_memo.created_at, memo.created_at);
        assert_eq!(updated_memo.updated_at, returned_memo.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_add_tags() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: None,
            favorite: None,
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
        assert_eq!(returned_memo.favorite, memo.favorite);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id).one(&db).await?.unwrap();
        assert_eq!(updated_memo.title, memo.title);
        assert_eq!(updated_memo.text, memo.text);
        assert_eq!(updated_memo.date, memo.date);
        assert_eq!(updated_memo.favorite, memo.favorite);
        assert_eq!(updated_memo.user_id, user.id);
        assert_eq!(updated_memo.created_at, memo.created_at);
        assert_eq!(updated_memo.updated_at, returned_memo.updated_at);

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
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_memo_tag(&db, memo.id, ambition_tag.id).await?;

        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: None,
            favorite: None,
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
        assert_eq!(returned_memo.favorite, memo.favorite);
        assert_eq!(returned_memo.user_id, user.id);
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(memo.id).one(&db).await?.unwrap();
        assert_eq!(updated_memo.title, memo.title);
        assert_eq!(updated_memo.text, memo.text);
        assert_eq!(updated_memo.date, memo.date);
        assert_eq!(updated_memo.favorite, memo.favorite);
        assert_eq!(updated_memo.user_id, user.id);
        assert_eq!(updated_memo.created_at, memo.created_at);
        assert_eq!(updated_memo.updated_at, returned_memo.updated_at);

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
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;
        let form = UpdateMemo {
            id: memo.id,
            title: None,
            text: None,
            date: None,
            favorite: None,
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

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_memo_tag(&db, memo.id, ambition_tag.id).await?;

        MemoMutation::delete(&db, memo.id, user.id).await?;

        let memo_in_db = memo::Entity::find_by_id(memo.id).one(&db).await?;
        assert!(memo_in_db.is_none());

        let memos_tags_in_db = memos_tags::Entity::find()
            .filter(memos_tags::Column::MemoId.eq(memo.id))
            .filter(memos_tags::Column::TagId.eq(ambition_tag.id))
            .one(&db)
            .await?;
        assert!(memos_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let memo = factory::memo(user.id).insert(&db).await?;

        let error = MemoMutation::delete(&db, memo.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }
}
