use crate::mindset_query::MindsetQuery;
use chrono::Utc;
use entities::{mindset, tag};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, Set, TransactionError, TransactionTrait,
};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewMindset {
    pub name: String,
    pub description: Option<String>,
    pub user_id: uuid::Uuid,
}

pub struct MindsetMutation;

impl MindsetMutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewMindset,
    ) -> Result<mindset::Model, TransactionError<DbErr>> {
        db.transaction::<_, mindset::Model, DbErr>(|txn| {
            Box::pin(async move {
                let mindset_id = uuid::Uuid::now_v7();
                let created_mindset = mindset::ActiveModel {
                    id: Set(mindset_id),
                    user_id: Set(form_data.user_id),
                    name: Set(form_data.name.to_owned()),
                    description: Set(form_data.description),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::now_v7()),
                    user_id: Set(form_data.user_id),
                    mindset_id: Set(Some(mindset_id)),
                    ..Default::default()
                }
                .insert(txn)
                .await?;

                Ok(created_mindset)
            })
        })
        .await
    }

    pub async fn update(
        db: &DbConn,
        mindset_id: uuid::Uuid,
        user_id: uuid::Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<mindset::Model, DbErr> {
        let mut mindset: mindset::ActiveModel =
            MindsetQuery::find_by_id_and_user_id(db, mindset_id, user_id)
                .await?
                .into();
        mindset.name = Set(name);
        mindset.description = Set(description);
        mindset.updated_at = Set(Utc::now().into());
        mindset.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        mindset_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        MindsetQuery::find_by_id_and_user_id(db, mindset_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn archive(
        db: &DbConn,
        mindset_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<mindset::Model, DbErr> {
        let mut mindset: mindset::ActiveModel =
            MindsetQuery::find_by_id_and_user_id(db, mindset_id, user_id)
                .await?
                .into();
        mindset.archived = Set(true);
        mindset.updated_at = Set(Utc::now().into());
        mindset.update(db).await
    }

    pub async fn unarchive(
        db: &DbConn,
        mindset_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<mindset::Model, DbErr> {
        let mut mindset: mindset::ActiveModel =
            MindsetQuery::find_by_id_and_user_id(db, mindset_id, user_id)
                .await?
                .into();
        mindset.archived = Set(false);
        mindset.updated_at = Set(Utc::now().into());
        mindset.update(db).await
    }

    // FIXME: Reduce query.
    pub async fn bulk_update_ordering(
        db: &DbConn,
        user_id: uuid::Uuid,
        ordering: Vec<uuid::Uuid>,
    ) -> Result<(), DbErr> {
        let mindsets = mindset::Entity::find()
            .filter(mindset::Column::UserId.eq(user_id))
            .filter(mindset::Column::Id.is_in(ordering.clone()))
            .all(db)
            .await?;
        for mindset in mindsets {
            let order = &ordering.iter().position(|id| id == &mindset.id);
            if let Some(order) = order {
                let mut mindset = mindset.into_active_model();
                mindset.ordering = Set(Some((order + 1) as i32));
                mindset.update(db).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::{
        db::init_db,
        factory::{self, *},
        settings::get_test_settings,
    };
    use types::CustomDbErr;

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let name = "Test MindsetMutation::create_with_tag".to_string();
        let description = Some("Dummy description".to_string());

        let form_data = NewMindset {
            name: name.clone(),
            description: description.clone(),
            user_id: user.id,
        };

        let res = MindsetMutation::create_with_tag(&db, form_data)
            .await
            .unwrap();
        assert_eq!(res.name, name);
        assert_eq!(res.description, description);
        assert_eq!(res.archived, false);
        assert_eq!(res.user_id, user.id);

        let mindset_in_db = mindset::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(mindset_in_db, res);

        let tag_in_db = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::MindsetId.eq(Some(res.id)))
            .filter(tag::Column::DesiredStateId.is_null())
            .filter(tag::Column::ActionId.is_null())
            .one(&db)
            .await?;
        assert!(tag_in_db.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let mindset = factory::mindset(user.id).insert(&db).await?;

        let new_name = "Test MindsetMutation::update_after".to_string();
        let new_description = Some("After update.".to_string());

        let res = MindsetMutation::update(
            &db,
            mindset.id,
            user.id,
            new_name.clone(),
            new_description.clone(),
        )
        .await?;
        assert_eq!(res.id, mindset.id);
        assert_eq!(res.name, new_name);
        assert_eq!(res.description, new_description);
        assert_eq!(res.archived, mindset.archived);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, mindset.created_at);
        assert!(res.updated_at > mindset.updated_at);

        let mindset_in_db = mindset::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(mindset_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let mindset = factory::mindset(user.id).insert(&db).await?;

        let new_name = "Test MindsetMutation::update_after".to_string();
        let new_description = Some("After update.".to_string());

        let error = MindsetMutation::update(
            &db,
            mindset.id,
            uuid::Uuid::now_v7(),
            new_name.clone(),
            new_description.clone(),
        )
        .await
        .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let (mindset, tag) = factory::mindset(user.id).insert_with_tag(&db).await?;

        MindsetMutation::delete(&db, mindset.id, user.id).await?;

        let mindset_in_db = mindset::Entity::find_by_id(mindset.id).one(&db).await?;
        assert!(mindset_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let mindset = factory::mindset(user.id).insert(&db).await?;

        let error = MindsetMutation::delete(&db, mindset.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn archive() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let mindset = factory::mindset(user.id).insert(&db).await?;

        let res = MindsetMutation::archive(&db, mindset.id, user.id).await?;
        assert_eq!(res.id, mindset.id);
        assert_eq!(res.name, mindset.name.clone());
        assert_eq!(res.description, mindset.description.clone());
        assert_eq!(res.archived, true);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, mindset.created_at);
        assert!(res.updated_at > mindset.updated_at);

        let mindset_in_db = mindset::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(mindset_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn archive_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let mindset = factory::mindset(user.id).insert(&db).await?;

        let error = MindsetMutation::archive(&db, mindset.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn unarchive() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let mindset = factory::mindset(user.id).archived(true).insert(&db).await?;

        let res = MindsetMutation::unarchive(&db, mindset.id, user.id).await?;
        assert_eq!(res.id, mindset.id);
        assert_eq!(res.name, mindset.name.clone());
        assert_eq!(res.description, mindset.description.clone());
        assert_eq!(res.archived, false);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, mindset.created_at);
        assert!(res.updated_at > mindset.updated_at);

        let mindset_in_db = mindset::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(mindset_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn unarchive_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let mindset = factory::mindset(user.id).archived(true).insert(&db).await?;

        let error = MindsetMutation::unarchive(&db, mindset.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn bulk_update_ordering() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let mindset_0 = factory::mindset(user.id).insert(&db).await?;
        let mindset_1 = factory::mindset(user.id).insert(&db).await?;
        let mindset_2 = factory::mindset(user.id).insert(&db).await?;

        let ordering = vec![mindset_0.id, mindset_1.id];

        MindsetMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let mindset_in_db_0 = mindset::Entity::find_by_id(mindset_0.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(mindset_in_db_0.ordering, Some(1));

        let mindset_in_db_1 = mindset::Entity::find_by_id(mindset_1.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(mindset_in_db_1.ordering, Some(2));

        let mindset_in_db_2 = mindset::Entity::find_by_id(mindset_2.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(mindset_in_db_2.ordering, None);

        Ok(())
    }

    #[actix_web::test]
    async fn bulk_update_ordering_no_modification_on_different_users_records() -> Result<(), DbErr>
    {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let another_users_mindset = factory::mindset(another_user.id).insert(&db).await?;

        let ordering = vec![another_users_mindset.id];

        MindsetMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let another_users_mindset_in_db = mindset::Entity::find_by_id(another_users_mindset.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(another_users_mindset_in_db.ordering, None);

        Ok(())
    }
}
