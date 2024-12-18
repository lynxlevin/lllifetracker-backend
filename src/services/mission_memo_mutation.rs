use crate::entities::{mission_memo, mission_memos_tags};
use chrono::Utc;
use sea_orm::{
    entity::prelude::*, DeriveColumn, EnumIter, QuerySelect, Set, TransactionError,
    TransactionTrait,
};

use super::mission_memo_query::MissionMemoQuery;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewMissionMemo {
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct UpdateMissionMemo {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub text: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub tag_ids: Option<Vec<uuid::Uuid>>,
    pub user_id: uuid::Uuid,
}

pub struct MissionMemoMutation;

impl MissionMemoMutation {
    pub async fn create(
        db: &DbConn,
        form_data: NewMissionMemo,
    ) -> Result<mission_memo::Model, TransactionError<DbErr>> {
        db.transaction::<_, mission_memo::Model, DbErr>(|txn| {
            Box::pin(async move {
                let now = Utc::now();
                let mission_memo_id = uuid::Uuid::new_v4();
                let created_mission_memo = mission_memo::ActiveModel {
                    id: Set(mission_memo_id),
                    user_id: Set(form_data.user_id),
                    title: Set(form_data.title.to_owned()),
                    text: Set(form_data.text.to_owned()),
                    date: Set(form_data.date),
                    archived: Set(false),
                    accomplished_at: Set(None),
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                }
                .insert(txn)
                .await?;

                for tag_id in form_data.tag_ids {
                    mission_memos_tags::ActiveModel {
                        mission_memo_id: Set(created_mission_memo.id),
                        tag_id: Set(tag_id),
                    }
                    .insert(txn)
                    .await?;
                }

                Ok(created_mission_memo)
            })
        })
        .await
    }

    pub async fn partial_update(
        db: &DbConn,
        form: UpdateMissionMemo,
    ) -> Result<mission_memo::Model, TransactionError<DbErr>> {
        let mission_memo_result =
            MissionMemoQuery::find_by_id_and_user_id(db, form.id, form.user_id).await;
        db.transaction::<_, mission_memo::Model, DbErr>(|txn| {
            Box::pin(async move {
                let mut mission_memo: mission_memo::ActiveModel = mission_memo_result?.into();
                if let Some(title) = form.title {
                    mission_memo.title = Set(title);
                }
                if let Some(text) = form.text {
                    mission_memo.text = Set(text);
                }
                if let Some(date) = form.date {
                    mission_memo.date = Set(date);
                }
                if let Some(tag_ids) = form.tag_ids {
                    let linked_tag_ids = mission_memos_tags::Entity::find()
                        .column_as(mission_memos_tags::Column::TagId, QueryAs::TagId)
                        .filter(mission_memos_tags::Column::MissionMemoId.eq(form.id))
                        .into_values::<uuid::Uuid, QueryAs>()
                        .all(txn)
                        .await?;

                    let tag_links_to_create: Vec<mission_memos_tags::ActiveModel> = tag_ids
                        .clone()
                        .into_iter()
                        .filter(|id| !linked_tag_ids.contains(id))
                        .map(|tag_id| mission_memos_tags::ActiveModel {
                            mission_memo_id: Set(form.id),
                            tag_id: Set(tag_id),
                        })
                        .collect();
                    mission_memos_tags::Entity::insert_many(tag_links_to_create)
                        .on_empty_do_nothing()
                        .exec(txn)
                        .await?;

                    let ids_to_delete: Vec<uuid::Uuid> = linked_tag_ids
                        .into_iter()
                        .filter(|linked_tag_id| !tag_ids.contains(linked_tag_id))
                        .collect();
                    if ids_to_delete.len() > 0 {
                        mission_memos_tags::Entity::delete_many()
                            .filter(mission_memos_tags::Column::MissionMemoId.eq(form.id))
                            .filter(mission_memos_tags::Column::TagId.is_in(ids_to_delete))
                            .exec(txn)
                            .await?;
                    }
                }
                mission_memo.updated_at = Set(Utc::now().into());
                mission_memo.update(txn).await
            })
        })
        .await
    }

    pub async fn delete(
        db: &DbConn,
        mission_memo_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        MissionMemoQuery::find_by_id_and_user_id(db, mission_memo_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn archive(
        db: &DbConn,
        mission_memo_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<mission_memo::Model, DbErr> {
        let mut mission_memo: mission_memo::ActiveModel =
            MissionMemoQuery::find_by_id_and_user_id(db, mission_memo_id, user_id)
                .await?
                .into();
        mission_memo.archived = Set(true);
        mission_memo.updated_at = Set(Utc::now().into());
        mission_memo.update(db).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use sea_orm::DbErr;

    use crate::test_utils;
    use crate::types::CustomDbErr;

    use super::*;

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
        let mission_memo_title = "New Mission Memo".to_string();
        let mission_memo_text = "This is a new mission memo for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();

        let form_data = NewMissionMemo {
            title: mission_memo_title.clone(),
            text: mission_memo_text.clone(),
            date: today,
            tag_ids: vec![tag_0.id, tag_1.id],
            user_id: user.id,
        };

        let returned_mission_memo = MissionMemoMutation::create(&db, form_data).await.unwrap();
        assert_eq!(returned_mission_memo.title, mission_memo_title.clone());
        assert_eq!(returned_mission_memo.text, mission_memo_text.clone());
        assert_eq!(returned_mission_memo.date, today);
        assert_eq!(returned_mission_memo.archived, false);
        assert_eq!(returned_mission_memo.accomplished_at, None);
        assert_eq!(returned_mission_memo.user_id, user.id);

        let created_mission_memo = mission_memo::Entity::find_by_id(returned_mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_mission_memo.title, mission_memo_title.clone());
        assert_eq!(created_mission_memo.text, mission_memo_text.clone());
        assert_eq!(created_mission_memo.date, today);
        assert_eq!(created_mission_memo.archived, false);
        assert_eq!(created_mission_memo.accomplished_at, None);
        assert_eq!(created_mission_memo.user_id, user.id);
        assert_eq!(
            created_mission_memo.created_at,
            returned_mission_memo.created_at
        );
        assert_eq!(
            created_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = mission_memos_tags::Entity::find()
            .column_as(mission_memos_tags::Column::TagId, QueryAs::TagId)
            .filter(mission_memos_tags::Column::MissionMemoId.eq(returned_mission_memo.id))
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
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission memo without tags".to_string(),
            user.id,
        )
        .await?;

        let form = UpdateMissionMemo {
            id: mission_memo.id,
            title: Some("Updated Mission Memo".to_string()),
            text: None,
            date: None,
            tag_ids: None,
            user_id: user.id,
        };

        let returned_mission_memo = MissionMemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_mission_memo.id, mission_memo.id);
        assert_eq!(returned_mission_memo.title, form.title.clone().unwrap());
        assert_eq!(returned_mission_memo.text, mission_memo.text);
        assert_eq!(returned_mission_memo.date, mission_memo.date);
        assert_eq!(returned_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            returned_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(returned_mission_memo.user_id, user.id);
        assert_eq!(returned_mission_memo.created_at, mission_memo.created_at);
        assert!(returned_mission_memo.updated_at > mission_memo.updated_at);

        let updated_mission_memo = mission_memo::Entity::find_by_id(mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_mission_memo.title, form.title.clone().unwrap());
        assert_eq!(updated_mission_memo.text, mission_memo.text);
        assert_eq!(updated_mission_memo.date, mission_memo.date);
        assert_eq!(updated_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            updated_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(updated_mission_memo.user_id, user.id);
        assert_eq!(updated_mission_memo.created_at, mission_memo.created_at);
        assert_eq!(
            updated_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_text() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission memo without tags".to_string(),
            user.id,
        )
        .await?;

        let form = UpdateMissionMemo {
            id: mission_memo.id,
            title: None,
            text: Some("Updated mission memo content.".to_string()),
            date: None,
            tag_ids: None,
            user_id: user.id,
        };

        let returned_mission_memo = MissionMemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_mission_memo.id, mission_memo.id);
        assert_eq!(returned_mission_memo.title, mission_memo.title);
        assert_eq!(returned_mission_memo.text, form.text.clone().unwrap());
        assert_eq!(returned_mission_memo.date, mission_memo.date);
        assert_eq!(returned_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            returned_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(returned_mission_memo.user_id, user.id);
        assert_eq!(returned_mission_memo.created_at, mission_memo.created_at);
        assert!(returned_mission_memo.updated_at > mission_memo.updated_at);

        let updated_mission_memo = mission_memo::Entity::find_by_id(mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_mission_memo.title, mission_memo.title);
        assert_eq!(updated_mission_memo.text, form.text.clone().unwrap());
        assert_eq!(updated_mission_memo.date, mission_memo.date);
        assert_eq!(updated_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            updated_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(updated_mission_memo.user_id, user.id);
        assert_eq!(updated_mission_memo.created_at, mission_memo.created_at);
        assert_eq!(
            updated_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_date() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission memo without tags".to_string(),
            user.id,
        )
        .await?;

        let form = UpdateMissionMemo {
            id: mission_memo.id,
            title: None,
            text: None,
            date: Some(chrono::Utc::now().with_year(1900).unwrap().date_naive()),
            tag_ids: None,
            user_id: user.id,
        };

        let returned_mission_memo = MissionMemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_mission_memo.id, mission_memo.id);
        assert_eq!(returned_mission_memo.title, mission_memo.title);
        assert_eq!(returned_mission_memo.text, mission_memo.text);
        assert_eq!(returned_mission_memo.date, form.date.unwrap());
        assert_eq!(returned_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            returned_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(returned_mission_memo.user_id, user.id);
        assert_eq!(returned_mission_memo.created_at, mission_memo.created_at);
        assert!(returned_mission_memo.updated_at > mission_memo.updated_at);

        let updated_mission_memo = mission_memo::Entity::find_by_id(mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_mission_memo.title, mission_memo.title);
        assert_eq!(updated_mission_memo.text, mission_memo.text);
        assert_eq!(updated_mission_memo.date, form.date.clone().unwrap());
        assert_eq!(updated_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            updated_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(updated_mission_memo.user_id, user.id);
        assert_eq!(updated_mission_memo.created_at, mission_memo.created_at);
        assert_eq!(
            updated_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_add_tags() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission memo without tags".to_string(),
            user.id,
        )
        .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;

        let form = UpdateMissionMemo {
            id: mission_memo.id,
            title: None,
            text: None,
            date: None,
            tag_ids: Some(vec![ambition_tag.id]),
            user_id: user.id,
        };

        let returned_mission_memo = MissionMemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_mission_memo.id, mission_memo.id);
        assert_eq!(returned_mission_memo.title, mission_memo.title);
        assert_eq!(returned_mission_memo.text, mission_memo.text);
        assert_eq!(returned_mission_memo.date, mission_memo.date);
        assert_eq!(returned_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            returned_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(returned_mission_memo.user_id, user.id);
        assert_eq!(returned_mission_memo.created_at, mission_memo.created_at);
        assert!(returned_mission_memo.updated_at > mission_memo.updated_at);

        let updated_mission_memo = mission_memo::Entity::find_by_id(mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_mission_memo.title, mission_memo.title);
        assert_eq!(updated_mission_memo.text, mission_memo.text);
        assert_eq!(updated_mission_memo.date, mission_memo.date);
        assert_eq!(updated_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            updated_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(updated_mission_memo.user_id, user.id);
        assert_eq!(updated_mission_memo.created_at, mission_memo.created_at);
        assert_eq!(
            updated_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = mission_memos_tags::Entity::find()
            .column_as(mission_memos_tags::Column::TagId, QueryAs::TagId)
            .filter(mission_memos_tags::Column::MissionMemoId.eq(returned_mission_memo.id))
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
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission memo without tags".to_string(),
            user.id,
        )
        .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        mission_memos_tags::ActiveModel {
            mission_memo_id: Set(mission_memo.id),
            tag_id: Set(ambition_tag.id),
        }
        .insert(&db)
        .await?;

        let form = UpdateMissionMemo {
            id: mission_memo.id,
            title: None,
            text: None,
            date: None,
            tag_ids: Some(vec![]),
            user_id: user.id,
        };

        let returned_mission_memo = MissionMemoMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_eq!(returned_mission_memo.id, mission_memo.id);
        assert_eq!(returned_mission_memo.title, mission_memo.title);
        assert_eq!(returned_mission_memo.text, mission_memo.text);
        assert_eq!(returned_mission_memo.date, mission_memo.date);
        assert_eq!(returned_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            returned_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(returned_mission_memo.user_id, user.id);
        assert_eq!(returned_mission_memo.created_at, mission_memo.created_at);
        assert!(returned_mission_memo.updated_at > mission_memo.updated_at);

        let updated_mission_memo = mission_memo::Entity::find_by_id(mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_mission_memo.title, mission_memo.title);
        assert_eq!(updated_mission_memo.text, mission_memo.text);
        assert_eq!(updated_mission_memo.date, mission_memo.date);
        assert_eq!(updated_mission_memo.archived, mission_memo.archived);
        assert_eq!(
            updated_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(updated_mission_memo.user_id, user.id);
        assert_eq!(updated_mission_memo.created_at, mission_memo.created_at);
        assert_eq!(
            updated_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = mission_memos_tags::Entity::find()
            .column_as(mission_memos_tags::Column::TagId, QueryAs::TagId)
            .filter(mission_memos_tags::Column::MissionMemoId.eq(returned_mission_memo.id))
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
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission memo without tags".to_string(),
            user.id,
        )
        .await?;
        let form = UpdateMissionMemo {
            id: mission_memo.id,
            title: None,
            text: None,
            date: None,
            tag_ids: None,
            user_id: uuid::Uuid::new_v4(),
        };

        let error = MissionMemoMutation::partial_update(&db, form)
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
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission memo to delete.".to_string(),
            user.id,
        )
        .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        mission_memos_tags::ActiveModel {
            mission_memo_id: Set(mission_memo.id),
            tag_id: Set(ambition_tag.id),
        }
        .insert(&db)
        .await?;

        MissionMemoMutation::delete(&db, mission_memo.id, user.id).await?;

        let mission_memo_in_db = mission_memo::Entity::find_by_id(mission_memo.id)
            .one(&db)
            .await?;
        assert!(mission_memo_in_db.is_none());

        let mission_memos_tags_in_db = mission_memos_tags::Entity::find()
            .filter(mission_memos_tags::Column::MissionMemoId.eq(mission_memo.id))
            .filter(mission_memos_tags::Column::TagId.eq(ambition_tag.id))
            .one(&db)
            .await?;
        assert!(mission_memos_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission memo to delete.".to_string(),
            user.id,
        )
        .await?;

        let error = MissionMemoMutation::delete(&db, mission_memo.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn archive() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo =
            test_utils::seed::create_mission_memo(&db, "Mission memo".to_string(), user.id).await?;

        let returned_mission_memo = MissionMemoMutation::archive(&db, mission_memo.id, user.id)
            .await
            .unwrap();
        assert_eq!(returned_mission_memo.id, mission_memo.id);
        assert_eq!(returned_mission_memo.title, mission_memo.title);
        assert_eq!(returned_mission_memo.text, mission_memo.text);
        assert_eq!(returned_mission_memo.date, mission_memo.date);
        assert_eq!(returned_mission_memo.archived, true);
        assert_eq!(
            returned_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(returned_mission_memo.user_id, user.id);
        assert_eq!(returned_mission_memo.created_at, mission_memo.created_at);
        assert!(returned_mission_memo.updated_at > mission_memo.updated_at);

        let updated_mission_memo = mission_memo::Entity::find_by_id(mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_mission_memo.title, mission_memo.title);
        assert_eq!(updated_mission_memo.text, mission_memo.text);
        assert_eq!(updated_mission_memo.date, mission_memo.date);
        assert_eq!(updated_mission_memo.archived, true);
        assert_eq!(
            updated_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(updated_mission_memo.user_id, user.id);
        assert_eq!(updated_mission_memo.created_at, mission_memo.created_at);
        assert_eq!(
            updated_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = mission_memos_tags::Entity::find()
            .column_as(mission_memos_tags::Column::TagId, QueryAs::TagId)
            .filter(mission_memos_tags::Column::MissionMemoId.eq(returned_mission_memo.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 0);

        Ok(())
    }

    #[actix_web::test]
    async fn archive_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo =
            test_utils::seed::create_mission_memo(&db, "Mission memo".to_string(), user.id).await?;

        let error = MissionMemoMutation::archive(&db, mission_memo.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }
}
