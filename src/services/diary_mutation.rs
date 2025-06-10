use entities::{diaries_tags, diary};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeriveColumn, EntityTrait, EnumIter,
    IntoActiveModel, ModelTrait, QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
};
use types::DiaryKey;

use super::diary_query::DiaryQuery;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewDiary {
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub score: Option<i16>,
    pub tag_ids: Vec<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct UpdateDiary {
    pub id: uuid::Uuid,
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub score: Option<i16>,
    pub tag_ids: Vec<uuid::Uuid>,
    pub user_id: uuid::Uuid,
    pub update_keys: Vec<DiaryKey>,
}

pub struct DiaryMutation;

impl DiaryMutation {
    pub async fn create(
        db: &DbConn,
        form_data: NewDiary,
    ) -> Result<diary::Model, TransactionError<DbErr>> {
        db.transaction::<_, diary::Model, DbErr>(|txn| {
            Box::pin(async move {
                let diary_id = uuid::Uuid::now_v7();
                let created_diary = diary::ActiveModel {
                    id: Set(diary_id),
                    user_id: Set(form_data.user_id),
                    text: Set(form_data.text),
                    date: Set(form_data.date),
                    score: Set(form_data.score),
                }
                .insert(txn)
                .await?;

                let tag_links_to_create: Vec<diaries_tags::ActiveModel> = form_data
                    .tag_ids
                    .clone()
                    .into_iter()
                    .map(|tag_id| diaries_tags::ActiveModel {
                        diary_id: Set(created_diary.id),
                        tag_id: Set(tag_id),
                    })
                    .collect();
                diaries_tags::Entity::insert_many(tag_links_to_create)
                    .on_empty_do_nothing()
                    .exec(txn)
                    .await?;

                Ok(created_diary)
            })
        })
        .await
    }

    pub async fn partial_update(
        db: &DbConn,
        form: UpdateDiary,
    ) -> Result<diary::Model, TransactionError<DbErr>> {
        let diary_result = DiaryQuery::find_by_id_and_user_id(db, form.id, form.user_id).await;
        db.transaction::<_, diary::Model, DbErr>(|txn| {
            Box::pin(async move {
                let mut diary = diary_result?.into_active_model();
                if form.update_keys.contains(&DiaryKey::Text) {
                    diary.text = Set(form.text);
                }
                if form.update_keys.contains(&DiaryKey::Date) {
                    diary.date = Set(form.date);
                }
                if form.update_keys.contains(&DiaryKey::Score) {
                    diary.score = Set(form.score);
                }
                if form.update_keys.contains(&DiaryKey::TagIds) {
                    let tag_ids = form.tag_ids;
                    let linked_tag_ids = diaries_tags::Entity::find()
                        .column_as(diaries_tags::Column::TagId, QueryAs::TagId)
                        .filter(diaries_tags::Column::DiaryId.eq(form.id))
                        .into_values::<uuid::Uuid, QueryAs>()
                        .all(txn)
                        .await?;

                    let tag_links_to_create: Vec<diaries_tags::ActiveModel> = tag_ids
                        .clone()
                        .into_iter()
                        .filter(|id| !linked_tag_ids.contains(id))
                        .map(|tag_id| diaries_tags::ActiveModel {
                            diary_id: Set(form.id),
                            tag_id: Set(tag_id),
                        })
                        .collect();
                    diaries_tags::Entity::insert_many(tag_links_to_create)
                        .on_empty_do_nothing()
                        .exec(txn)
                        .await?;

                    let ids_to_delete: Vec<uuid::Uuid> = linked_tag_ids
                        .into_iter()
                        .filter(|linked_tag_id| !tag_ids.contains(linked_tag_id))
                        .collect();
                    if ids_to_delete.len() > 0 {
                        diaries_tags::Entity::delete_many()
                            .filter(diaries_tags::Column::DiaryId.eq(form.id))
                            .filter(diaries_tags::Column::TagId.is_in(ids_to_delete))
                            .exec(txn)
                            .await?;
                    }
                }
                diary.update(txn).await
            })
        })
        .await
    }

    pub async fn delete(
        db: &DbConn,
        diary_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        DiaryQuery::find_by_id_and_user_id(db, diary_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::DbErr;

    use ::types::CustomDbErr;
    use common::{
        db::init_db,
        factory::{self, *},
        settings::get_test_settings,
    };

    use super::*;

    #[actix_web::test]
    async fn create() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let (_, tag_0) = factory::action(user.id)
            .name("action_0".to_string())
            .insert_with_tag(&db)
            .await?;
        let (_, tag_1) = factory::action(user.id)
            .name("action_1".to_string())
            .insert_with_tag(&db)
            .await?;

        let diary_text = "This is a new diary for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();
        let diary_score = 3;

        let form_data = NewDiary {
            text: Some(diary_text.clone()),
            date: today,
            score: Some(diary_score),
            tag_ids: vec![tag_0.id, tag_1.id],
            user_id: user.id,
        };

        let res = DiaryMutation::create(&db, form_data).await.unwrap();
        assert_eq!(res.text, Some(diary_text.clone()));
        assert_eq!(res.date, today);
        assert_eq!(res.score, Some(diary_score.clone()));
        assert_eq!(res.user_id, user.id);

        let diary_in_db = diary::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(diary_in_db, res);

        let tag_query =
            diaries_tags::Entity::find().filter(diaries_tags::Column::DiaryId.eq(res.id));
        assert_eq!(tag_query.clone().all(&db).await?.len(), 2);
        assert!(tag_query
            .clone()
            .filter(diaries_tags::Column::TagId.eq(tag_0.id))
            .one(&db)
            .await?
            .is_some());
        assert!(tag_query
            .clone()
            .filter(diaries_tags::Column::TagId.eq(tag_1.id))
            .one(&db)
            .await?
            .is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_text() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let (_, tag) = factory::action(user.id).insert_with_tag(&db).await?;

        let form = UpdateDiary {
            id: diary.id,
            text: Some("Updated diary content.".to_string()),
            date: diary.date - chrono::TimeDelta::days(1),
            score: None,
            tag_ids: vec![tag.id],
            user_id: user.id,
            update_keys: vec![DiaryKey::Text],
        };

        let res = DiaryMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(res.id, diary.id);
        assert_eq!(res.text, form.text.clone());
        assert_eq!(res.date, diary.date);
        assert_eq!(res.score, diary.score);
        assert_eq!(res.user_id, user.id);

        let diary_in_db = diary::Entity::find_by_id(diary.id).one(&db).await?.unwrap();
        assert_eq!(diary_in_db, res);

        let tag_link = diaries_tags::Entity::find()
            .filter(diaries_tags::Column::DiaryId.eq(diary.id))
            .all(&db)
            .await?;
        assert_eq!(tag_link.len(), 0);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_date() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let (_, tag) = factory::action(user.id).insert_with_tag(&db).await?;

        let form = UpdateDiary {
            id: diary.id,
            text: Some("Updated diary content.".to_string()),
            date: diary.date - chrono::TimeDelta::days(1),
            score: None,
            tag_ids: vec![tag.id],
            user_id: user.id,
            update_keys: vec![DiaryKey::Date],
        };

        let res = DiaryMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(res.id, diary.id);
        assert_eq!(res.text, diary.text);
        assert_eq!(res.date, form.date);
        assert_eq!(res.score, diary.score);
        assert_eq!(res.user_id, user.id);

        let diary_in_db = diary::Entity::find_by_id(diary.id).one(&db).await?.unwrap();
        assert_eq!(diary_in_db, res);

        let tag_link = diaries_tags::Entity::find()
            .filter(diaries_tags::Column::DiaryId.eq(diary.id))
            .all(&db)
            .await?;
        assert_eq!(tag_link.len(), 0);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_score() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let (_, tag) = factory::action(user.id).insert_with_tag(&db).await?;

        let form = UpdateDiary {
            id: diary.id,
            text: Some("Updated diary content.".to_string()),
            date: diary.date - chrono::TimeDelta::days(1),
            score: None,
            tag_ids: vec![tag.id],
            user_id: user.id,
            update_keys: vec![DiaryKey::Score],
        };

        let res = DiaryMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(res.id, diary.id);
        assert_eq!(res.text, diary.text);
        assert_eq!(res.date, diary.date);
        assert_eq!(res.score, form.score);
        assert_eq!(res.user_id, user.id);

        let diary_in_db = diary::Entity::find_by_id(diary.id).one(&db).await?.unwrap();
        assert_eq!(diary_in_db, res);

        let tag_link = diaries_tags::Entity::find()
            .filter(diaries_tags::Column::DiaryId.eq(diary.id))
            .all(&db)
            .await?;
        assert_eq!(tag_link.len(), 0);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_add_tags() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let (_, tag) = factory::action(user.id).insert_with_tag(&db).await?;

        let form = UpdateDiary {
            id: diary.id,
            text: Some("Updated diary content.".to_string()),
            date: diary.date - chrono::TimeDelta::days(1),
            score: None,
            tag_ids: vec![tag.id],
            user_id: user.id,
            update_keys: vec![DiaryKey::TagIds],
        };

        let res = DiaryMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(res.id, diary.id);
        assert_eq!(res.text, diary.text);
        assert_eq!(res.date, diary.date);
        assert_eq!(res.score, diary.score);
        assert_eq!(res.user_id, user.id);

        let diary_in_db = diary::Entity::find_by_id(diary.id).one(&db).await?.unwrap();
        assert_eq!(diary_in_db, res);

        let tag_link = diaries_tags::Entity::find()
            .filter(diaries_tags::Column::DiaryId.eq(diary.id))
            .all(&db)
            .await?;
        assert_eq!(tag_link.len(), 1);
        assert_eq!(tag_link[0].tag_id, tag.id);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_remove_tags() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let (_, tag) = factory::action(user.id).insert_with_tag(&db).await?;
        factory::link_diary_tag(&db, diary.id, tag.id).await?;

        let form = UpdateDiary {
            id: diary.id,
            text: Some("Updated diary content.".to_string()),
            date: diary.date - chrono::TimeDelta::days(1),
            score: None,
            tag_ids: vec![],
            user_id: user.id,
            update_keys: vec![DiaryKey::TagIds],
        };

        let res = DiaryMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(res.id, diary.id);
        assert_eq!(res.text, diary.text);
        assert_eq!(res.date, diary.date);
        assert_eq!(res.score, diary.score);
        assert_eq!(res.user_id, user.id);

        let diary_in_db = diary::Entity::find_by_id(diary.id).one(&db).await?.unwrap();
        assert_eq!(diary_in_db, res);

        let tag_link = diaries_tags::Entity::find()
            .filter(diaries_tags::Column::DiaryId.eq(diary.id))
            .all(&db)
            .await?;
        assert_eq!(tag_link.len(), 0);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let form = UpdateDiary {
            id: diary.id,
            text: None,
            date: diary.date,
            score: None,
            tag_ids: vec![],
            user_id: uuid::Uuid::now_v7(),
            update_keys: vec![],
        };

        let error = DiaryMutation::partial_update(&db, form).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            TransactionError::Transaction(DbErr::Custom(CustomDbErr::NotFound.to_string()))
                .to_string(),
        );

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let (_, tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_diary_tag(&db, diary.id, tag.id).await?;

        DiaryMutation::delete(&db, diary.id, user.id).await?;

        let diary_in_db = diary::Entity::find_by_id(diary.id).one(&db).await?;
        assert!(diary_in_db.is_none());

        let diaries_tags_in_db = diaries_tags::Entity::find()
            .filter(diaries_tags::Column::DiaryId.eq(diary.id))
            .filter(diaries_tags::Column::TagId.eq(tag.id))
            .one(&db)
            .await?;
        assert!(diaries_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;

        let error = DiaryMutation::delete(&db, diary.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }
}
