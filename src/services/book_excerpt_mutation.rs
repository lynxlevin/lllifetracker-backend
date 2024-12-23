use crate::entities::{book_excerpt, book_excerpts_tags};
use chrono::Utc;
use sea_orm::{
    entity::prelude::*, DeriveColumn, EnumIter, QuerySelect, Set, TransactionError,
    TransactionTrait,
};

use super::book_excerpt_query::BookExcerptQuery;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewBookExcerpt {
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct UpdateBookExcerpt {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub page_number: Option<i16>,
    pub text: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub tag_ids: Option<Vec<uuid::Uuid>>,
    pub user_id: uuid::Uuid,
}

pub struct BookExcerptMutation;

impl BookExcerptMutation {
    pub async fn create(
        db: &DbConn,
        form_data: NewBookExcerpt,
    ) -> Result<book_excerpt::Model, TransactionError<DbErr>> {
        db.transaction::<_, book_excerpt::Model, DbErr>(|txn| {
            Box::pin(async move {
                let now = Utc::now();
                let book_excerpt_id = uuid::Uuid::new_v4();
                let created_book_excerpt = book_excerpt::ActiveModel {
                    id: Set(book_excerpt_id),
                    user_id: Set(form_data.user_id),
                    title: Set(form_data.title.to_owned()),
                    page_number: Set(form_data.page_number),
                    text: Set(form_data.text.to_owned()),
                    date: Set(form_data.date),
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                }
                .insert(txn)
                .await?;

                for tag_id in form_data.tag_ids {
                    book_excerpts_tags::ActiveModel {
                        book_excerpt_id: Set(created_book_excerpt.id),
                        tag_id: Set(tag_id),
                    }
                    .insert(txn)
                    .await?;
                }

                Ok(created_book_excerpt)
            })
        })
        .await
    }

    pub async fn partial_update(
        db: &DbConn,
        form: UpdateBookExcerpt,
    ) -> Result<book_excerpt::Model, TransactionError<DbErr>> {
        let book_excerpt_result =
            BookExcerptQuery::find_by_id_and_user_id(db, form.id, form.user_id).await;
        db.transaction::<_, book_excerpt::Model, DbErr>(|txn| {
            Box::pin(async move {
                let mut book_excerpt: book_excerpt::ActiveModel = book_excerpt_result?.into();
                if let Some(title) = form.title {
                    book_excerpt.title = Set(title);
                }
                if let Some(page_number) = form.page_number {
                    book_excerpt.page_number = Set(page_number);
                }
                if let Some(text) = form.text {
                    book_excerpt.text = Set(text);
                }
                if let Some(date) = form.date {
                    book_excerpt.date = Set(date);
                }
                if let Some(tag_ids) = form.tag_ids {
                    let linked_tag_ids = book_excerpts_tags::Entity::find()
                        .column_as(book_excerpts_tags::Column::TagId, QueryAs::TagId)
                        .filter(book_excerpts_tags::Column::BookExcerptId.eq(form.id))
                        .into_values::<uuid::Uuid, QueryAs>()
                        .all(txn)
                        .await?;

                    let tag_links_to_create: Vec<book_excerpts_tags::ActiveModel> = tag_ids
                        .clone()
                        .into_iter()
                        .filter(|id| !linked_tag_ids.contains(id))
                        .map(|tag_id| book_excerpts_tags::ActiveModel {
                            book_excerpt_id: Set(form.id),
                            tag_id: Set(tag_id),
                        })
                        .collect();
                    book_excerpts_tags::Entity::insert_many(tag_links_to_create)
                        .on_empty_do_nothing()
                        .exec(txn)
                        .await?;

                    let ids_to_delete: Vec<uuid::Uuid> = linked_tag_ids
                        .into_iter()
                        .filter(|linked_tag_id| !tag_ids.contains(linked_tag_id))
                        .collect();
                    if ids_to_delete.len() > 0 {
                        book_excerpts_tags::Entity::delete_many()
                            .filter(book_excerpts_tags::Column::BookExcerptId.eq(form.id))
                            .filter(book_excerpts_tags::Column::TagId.is_in(ids_to_delete))
                            .exec(txn)
                            .await?;
                    }
                }
                book_excerpt.updated_at = Set(Utc::now().into());
                book_excerpt.update(txn).await
            })
        })
        .await
    }

    pub async fn delete(
        db: &DbConn,
        book_excerpt_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        BookExcerptQuery::find_by_id_and_user_id(db, book_excerpt_id, user_id)
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

    use crate::test_utils;
    use crate::types::CustomDbErr;

    use super::*;

    /// Asserts equality of the following fields.
    /// ```
    /// id
    /// title
    /// page_number
    /// text
    /// date
    /// user_id
    /// created_at
    /// updated_at: actual > expected
    /// ```
    fn assert_updated(actual: &book_excerpt::Model, expected: &book_excerpt::Model) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.title, expected.title);
        assert_eq!(actual.page_number, expected.page_number);
        assert_eq!(actual.text, expected.text);
        assert_eq!(actual.date, expected.date);
        assert_eq!(actual.user_id, expected.user_id);
        assert_eq!(actual.created_at, expected.created_at);
        assert!(actual.updated_at > expected.updated_at);
    }

    #[actix_web::test]
    async fn create() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (_, tag_0) =
            test_utils::seed::create_action_and_tag(&db, "action_0".to_string(), None, user.id)
                .await?;
        let (_, tag_1) =
            test_utils::seed::create_action_and_tag(&db, "action_1".to_string(), None, user.id)
                .await?;
        let book_excerpt_title = "New Book Excerpt".to_string();
        let page_number = 13;
        let book_excerpt_text = "This is a new Book Excerpt for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();

        let form_data = NewBookExcerpt {
            title: book_excerpt_title.clone(),
            page_number: page_number,
            text: book_excerpt_text.clone(),
            date: today,
            tag_ids: vec![tag_0.id, tag_1.id],
            user_id: user.id,
        };

        let returned_book_excerpt = BookExcerptMutation::create(&db, form_data).await.unwrap();
        assert_eq!(returned_book_excerpt.title, book_excerpt_title.clone());
        assert_eq!(returned_book_excerpt.page_number, page_number);
        assert_eq!(returned_book_excerpt.text, book_excerpt_text.clone());
        assert_eq!(returned_book_excerpt.date, today);
        assert_eq!(returned_book_excerpt.user_id, user.id);

        let created_book_excerpt = book_excerpt::Entity::find_by_id(returned_book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_book_excerpt.title, book_excerpt_title.clone());
        assert_eq!(created_book_excerpt.page_number, page_number);
        assert_eq!(created_book_excerpt.text, book_excerpt_text.clone());
        assert_eq!(created_book_excerpt.date, today);
        assert_eq!(created_book_excerpt.user_id, user.id);
        assert_eq!(
            created_book_excerpt.created_at,
            returned_book_excerpt.created_at
        );
        assert_eq!(
            created_book_excerpt.updated_at,
            returned_book_excerpt.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = book_excerpts_tags::Entity::find()
            .column_as(book_excerpts_tags::Column::TagId, QueryAs::TagId)
            .filter(book_excerpts_tags::Column::BookExcerptId.eq(returned_book_excerpt.id))
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
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt without tags".to_string(),
            user.id,
        )
        .await?;

        let form = UpdateBookExcerpt {
            id: book_excerpt.id,
            title: Some("Updated Book Excerpt".to_string()),
            page_number: None,
            text: None,
            date: None,
            tag_ids: None,
            user_id: user.id,
        };
        let mut expected = book_excerpt.clone();
        expected.title = form.title.clone().unwrap();

        let returned_book_excerpt = BookExcerptMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_book_excerpt, &expected);

        let updated_book_excerpt = book_excerpt::Entity::find_by_id(book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_book_excerpt, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_page_number() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt without tags".to_string(),
            user.id,
        )
        .await?;

        let form = UpdateBookExcerpt {
            id: book_excerpt.id,
            title: None,
            page_number: Some(134),
            text: None,
            date: None,
            tag_ids: None,
            user_id: user.id,
        };
        let mut expected = book_excerpt.clone();
        expected.page_number = form.page_number.unwrap();

        let returned_book_excerpt = BookExcerptMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_book_excerpt, &expected);

        let updated_book_excerpt = book_excerpt::Entity::find_by_id(book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_book_excerpt, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_text() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt without tags".to_string(),
            user.id,
        )
        .await?;

        let form = UpdateBookExcerpt {
            id: book_excerpt.id,
            title: None,
            page_number: None,
            text: Some("Updated Book Excerpt content.".to_string()),
            date: None,
            tag_ids: None,
            user_id: user.id,
        };

        let mut expected = book_excerpt.clone();
        expected.text = form.text.clone().unwrap();

        let returned_book_excerpt = BookExcerptMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_book_excerpt, &expected);

        let updated_book_excerpt = book_excerpt::Entity::find_by_id(book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_book_excerpt, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_date() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt without tags".to_string(),
            user.id,
        )
        .await?;

        let form = UpdateBookExcerpt {
            id: book_excerpt.id,
            title: None,
            page_number: None,
            text: None,
            date: Some(chrono::Utc::now().with_year(1900).unwrap().date_naive()),
            tag_ids: None,
            user_id: user.id,
        };

        let mut expected = book_excerpt.clone();
        expected.date = form.date.unwrap();

        let returned_book_excerpt = BookExcerptMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_book_excerpt, &expected);

        let updated_book_excerpt = book_excerpt::Entity::find_by_id(book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_book_excerpt, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_add_tags() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt without tags".to_string(),
            user.id,
        )
        .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;

        let form = UpdateBookExcerpt {
            id: book_excerpt.id,
            title: None,
            page_number: None,
            text: None,
            date: None,
            tag_ids: Some(vec![ambition_tag.id]),
            user_id: user.id,
        };

        let expected = book_excerpt.clone();

        let returned_book_excerpt = BookExcerptMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_book_excerpt, &expected);

        let updated_book_excerpt = book_excerpt::Entity::find_by_id(book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_book_excerpt, &expected);

        let linked_tag_ids: Vec<uuid::Uuid> = book_excerpts_tags::Entity::find()
            .column_as(book_excerpts_tags::Column::TagId, QueryAs::TagId)
            .filter(book_excerpts_tags::Column::BookExcerptId.eq(returned_book_excerpt.id))
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
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt without tags".to_string(),
            user.id,
        )
        .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        book_excerpts_tags::ActiveModel {
            book_excerpt_id: Set(book_excerpt.id),
            tag_id: Set(ambition_tag.id),
        }
        .insert(&db)
        .await?;

        let form = UpdateBookExcerpt {
            id: book_excerpt.id,
            title: None,
            page_number: None,
            text: None,
            date: None,
            tag_ids: Some(vec![]),
            user_id: user.id,
        };

        let expected = book_excerpt.clone();

        let returned_book_excerpt = BookExcerptMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_book_excerpt, &expected);

        let updated_book_excerpt = book_excerpt::Entity::find_by_id(book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_book_excerpt, &expected);

        let linked_tag_ids: Vec<uuid::Uuid> = book_excerpts_tags::Entity::find()
            .column_as(book_excerpts_tags::Column::TagId, QueryAs::TagId)
            .filter(book_excerpts_tags::Column::BookExcerptId.eq(returned_book_excerpt.id))
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
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt without tags".to_string(),
            user.id,
        )
        .await?;
        let form = UpdateBookExcerpt {
            id: book_excerpt.id,
            title: None,
            page_number: None,
            text: None,
            date: None,
            tag_ids: None,
            user_id: uuid::Uuid::new_v4(),
        };

        let error = BookExcerptMutation::partial_update(&db, form)
            .await
            .unwrap_err();
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
        let user = test_utils::seed::create_active_user(&db).await?;
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt to delete.".to_string(),
            user.id,
        )
        .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        book_excerpts_tags::ActiveModel {
            book_excerpt_id: Set(book_excerpt.id),
            tag_id: Set(ambition_tag.id),
        }
        .insert(&db)
        .await?;

        BookExcerptMutation::delete(&db, book_excerpt.id, user.id).await?;

        let book_excerpt_in_db = book_excerpt::Entity::find_by_id(book_excerpt.id)
            .one(&db)
            .await?;
        assert!(book_excerpt_in_db.is_none());

        let book_excerpts_tags_in_db = book_excerpts_tags::Entity::find()
            .filter(book_excerpts_tags::Column::BookExcerptId.eq(book_excerpt.id))
            .filter(book_excerpts_tags::Column::TagId.eq(ambition_tag.id))
            .one(&db)
            .await?;
        assert!(book_excerpts_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "Book Excerpt to delete.".to_string(),
            user.id,
        )
        .await?;

        let error = BookExcerptMutation::delete(&db, book_excerpt.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }
}
