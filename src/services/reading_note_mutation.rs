use chrono::Utc;
use entities::{reading_note, reading_notes_tags};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeriveColumn, EntityTrait, EnumIter, ModelTrait,
    QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
};

use super::reading_note_query::ReadingNoteQuery;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewReadingNote {
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct UpdateReadingNote {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub page_number: Option<i16>,
    pub text: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub tag_ids: Option<Vec<uuid::Uuid>>,
    pub user_id: uuid::Uuid,
}

pub struct ReadingNoteMutation;

impl ReadingNoteMutation {
    pub async fn create(
        db: &DbConn,
        form_data: NewReadingNote,
    ) -> Result<reading_note::Model, TransactionError<DbErr>> {
        db.transaction::<_, reading_note::Model, DbErr>(|txn| {
            Box::pin(async move {
                let now = Utc::now();
                let reading_note_id = uuid::Uuid::now_v7();
                let created_reading_note = reading_note::ActiveModel {
                    id: Set(reading_note_id),
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
                    reading_notes_tags::ActiveModel {
                        reading_note_id: Set(created_reading_note.id),
                        tag_id: Set(tag_id),
                    }
                    .insert(txn)
                    .await?;
                }

                Ok(created_reading_note)
            })
        })
        .await
    }

    pub async fn partial_update(
        db: &DbConn,
        form: UpdateReadingNote,
    ) -> Result<reading_note::Model, TransactionError<DbErr>> {
        let reading_note_result =
            ReadingNoteQuery::find_by_id_and_user_id(db, form.id, form.user_id).await;
        db.transaction::<_, reading_note::Model, DbErr>(|txn| {
            Box::pin(async move {
                let mut reading_note: reading_note::ActiveModel = reading_note_result?.into();
                if let Some(title) = form.title {
                    reading_note.title = Set(title);
                }
                if let Some(page_number) = form.page_number {
                    reading_note.page_number = Set(page_number);
                }
                if let Some(text) = form.text {
                    reading_note.text = Set(text);
                }
                if let Some(date) = form.date {
                    reading_note.date = Set(date);
                }
                if let Some(tag_ids) = form.tag_ids {
                    let linked_tag_ids = reading_notes_tags::Entity::find()
                        .column_as(reading_notes_tags::Column::TagId, QueryAs::TagId)
                        .filter(reading_notes_tags::Column::ReadingNoteId.eq(form.id))
                        .into_values::<uuid::Uuid, QueryAs>()
                        .all(txn)
                        .await?;

                    let tag_links_to_create: Vec<reading_notes_tags::ActiveModel> = tag_ids
                        .clone()
                        .into_iter()
                        .filter(|id| !linked_tag_ids.contains(id))
                        .map(|tag_id| reading_notes_tags::ActiveModel {
                            reading_note_id: Set(form.id),
                            tag_id: Set(tag_id),
                        })
                        .collect();
                    reading_notes_tags::Entity::insert_many(tag_links_to_create)
                        .on_empty_do_nothing()
                        .exec(txn)
                        .await?;

                    let ids_to_delete: Vec<uuid::Uuid> = linked_tag_ids
                        .into_iter()
                        .filter(|linked_tag_id| !tag_ids.contains(linked_tag_id))
                        .collect();
                    if ids_to_delete.len() > 0 {
                        reading_notes_tags::Entity::delete_many()
                            .filter(reading_notes_tags::Column::ReadingNoteId.eq(form.id))
                            .filter(reading_notes_tags::Column::TagId.is_in(ids_to_delete))
                            .exec(txn)
                            .await?;
                    }
                }
                reading_note.updated_at = Set(Utc::now().into());
                reading_note.update(txn).await
            })
        })
        .await
    }

    pub async fn delete(
        db: &DbConn,
        reading_note_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        ReadingNoteQuery::find_by_id_and_user_id(db, reading_note_id, user_id)
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
    use common::factory::{self, *};
    use test_utils;

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
    fn assert_updated(actual: &reading_note::Model, expected: &reading_note::Model) {
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
        let user = factory::user().insert(&db).await?;
        let (_, tag_0) = factory::action(user.id)
            .name("action_0".to_string())
            .insert_with_tag(&db)
            .await?;
        let (_, tag_1) = factory::action(user.id)
            .name("action_1".to_string())
            .insert_with_tag(&db)
            .await?;

        let reading_note_title = "New Reading Note".to_string();
        let page_number = 13;
        let reading_note_text = "This is a new Reading Note for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();

        let form_data = NewReadingNote {
            title: reading_note_title.clone(),
            page_number: page_number,
            text: reading_note_text.clone(),
            date: today,
            tag_ids: vec![tag_0.id, tag_1.id],
            user_id: user.id,
        };

        let res = ReadingNoteMutation::create(&db, form_data).await.unwrap();
        assert_eq!(res.title, reading_note_title.clone());
        assert_eq!(res.page_number, page_number);
        assert_eq!(res.text, reading_note_text.clone());
        assert_eq!(res.date, today);
        assert_eq!(res.user_id, user.id);

        let reading_note_in_db = reading_note::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(reading_note_in_db, res);

        let linked_tag_ids: Vec<uuid::Uuid> = reading_notes_tags::Entity::find()
            .column_as(reading_notes_tags::Column::TagId, QueryAs::TagId)
            .filter(reading_notes_tags::Column::ReadingNoteId.eq(res.id))
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
        let reading_note = factory::reading_note(user.id).insert(&db).await?;

        let form = UpdateReadingNote {
            id: reading_note.id,
            title: Some("Updated Reading Note".to_string()),
            page_number: None,
            text: None,
            date: None,
            tag_ids: None,
            user_id: user.id,
        };
        let mut expected = reading_note.clone();
        expected.title = form.title.clone().unwrap();

        let res = ReadingNoteMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&res, &expected);

        let reading_note_in_db = reading_note::Entity::find_by_id(reading_note.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(reading_note_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_page_number() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let reading_note = factory::reading_note(user.id).insert(&db).await?;

        let form = UpdateReadingNote {
            id: reading_note.id,
            title: None,
            page_number: Some(134),
            text: None,
            date: None,
            tag_ids: None,
            user_id: user.id,
        };
        let mut expected = reading_note.clone();
        expected.page_number = form.page_number.unwrap();

        let res = ReadingNoteMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&res, &expected);

        let reading_note_in_db = reading_note::Entity::find_by_id(reading_note.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(reading_note_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_text() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let reading_note = factory::reading_note(user.id).insert(&db).await?;

        let form = UpdateReadingNote {
            id: reading_note.id,
            title: None,
            page_number: None,
            text: Some("Updated Reading Note content.".to_string()),
            date: None,
            tag_ids: None,
            user_id: user.id,
        };

        let mut expected = reading_note.clone();
        expected.text = form.text.clone().unwrap();

        let res = ReadingNoteMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&res, &expected);

        let reading_note_in_db = reading_note::Entity::find_by_id(reading_note.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(reading_note_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_date() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let reading_note = factory::reading_note(user.id).insert(&db).await?;

        let form = UpdateReadingNote {
            id: reading_note.id,
            title: None,
            page_number: None,
            text: None,
            date: Some(chrono::Utc::now().with_year(1900).unwrap().date_naive()),
            tag_ids: None,
            user_id: user.id,
        };

        let mut expected = reading_note.clone();
        expected.date = form.date.unwrap();

        let res = ReadingNoteMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&res, &expected);

        let reading_note_in_db = reading_note::Entity::find_by_id(reading_note.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(reading_note_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_add_tags() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let reading_note = factory::reading_note(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

        let form = UpdateReadingNote {
            id: reading_note.id,
            title: None,
            page_number: None,
            text: None,
            date: None,
            tag_ids: Some(vec![ambition_tag.id]),
            user_id: user.id,
        };

        let expected = reading_note.clone();

        let res = ReadingNoteMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&res, &expected);

        let reading_note_in_db = reading_note::Entity::find_by_id(reading_note.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(reading_note_in_db, res);

        let linked_tag_ids: Vec<uuid::Uuid> = reading_notes_tags::Entity::find()
            .column_as(reading_notes_tags::Column::TagId, QueryAs::TagId)
            .filter(reading_notes_tags::Column::ReadingNoteId.eq(res.id))
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
        let reading_note = factory::reading_note(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_reading_note_tag(&db, reading_note.id, ambition_tag.id).await?;

        let form = UpdateReadingNote {
            id: reading_note.id,
            title: None,
            page_number: None,
            text: None,
            date: None,
            tag_ids: Some(vec![]),
            user_id: user.id,
        };

        let expected = reading_note.clone();

        let res = ReadingNoteMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&res, &expected);

        let reading_note_in_db = reading_note::Entity::find_by_id(reading_note.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(reading_note_in_db, res);

        let linked_tag_ids: Vec<uuid::Uuid> = reading_notes_tags::Entity::find()
            .column_as(reading_notes_tags::Column::TagId, QueryAs::TagId)
            .filter(reading_notes_tags::Column::ReadingNoteId.eq(res.id))
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
        let reading_note = factory::reading_note(user.id).insert(&db).await?;
        let form = UpdateReadingNote {
            id: reading_note.id,
            title: None,
            page_number: None,
            text: None,
            date: None,
            tag_ids: None,
            user_id: uuid::Uuid::now_v7(),
        };

        let error = ReadingNoteMutation::partial_update(&db, form)
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
        let user = factory::user().insert(&db).await?;
        let reading_note = factory::reading_note(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_reading_note_tag(&db, reading_note.id, ambition_tag.id).await?;

        ReadingNoteMutation::delete(&db, reading_note.id, user.id).await?;

        let reading_note_in_db = reading_note::Entity::find_by_id(reading_note.id)
            .one(&db)
            .await?;
        assert!(reading_note_in_db.is_none());

        let reading_notes_tags_in_db = reading_notes_tags::Entity::find()
            .filter(reading_notes_tags::Column::ReadingNoteId.eq(reading_note.id))
            .filter(reading_notes_tags::Column::TagId.eq(ambition_tag.id))
            .one(&db)
            .await?;
        assert!(reading_notes_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let reading_note = factory::reading_note(user.id).insert(&db).await?;

        let error = ReadingNoteMutation::delete(&db, reading_note.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }
}
