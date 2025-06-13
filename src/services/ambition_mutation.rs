use chrono::Utc;
use entities::{ambition, tag};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, Set, TransactionError, TransactionTrait,
};

use super::ambition_query::AmbitionQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewAmbition {
    pub name: String,
    pub description: Option<String>,
    pub user_id: uuid::Uuid,
}

pub struct AmbitionMutation;

impl AmbitionMutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewAmbition,
    ) -> Result<ambition::Model, TransactionError<DbErr>> {
        db.transaction::<_, ambition::Model, DbErr>(|txn| {
            Box::pin(async move {
                let ambition_id = uuid::Uuid::now_v7();
                let created_ambition = ambition::ActiveModel {
                    id: Set(ambition_id),
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
                    ambition_id: Set(Some(ambition_id)),
                    ..Default::default()
                }
                .insert(txn)
                .await?;

                Ok(created_ambition)
            })
        })
        .await
    }

    pub async fn update(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<ambition::Model, DbErr> {
        let mut ambition: ambition::ActiveModel =
            AmbitionQuery::find_by_id_and_user_id(db, ambition_id, user_id)
                .await?
                .into();
        ambition.name = Set(name);
        ambition.description = Set(description);
        ambition.updated_at = Set(Utc::now().into());
        ambition.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        AmbitionQuery::find_by_id_and_user_id(db, ambition_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn archive(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<ambition::Model, DbErr> {
        let mut ambition: ambition::ActiveModel =
            AmbitionQuery::find_by_id_and_user_id(db, ambition_id, user_id)
                .await?
                .into();
        ambition.archived = Set(true);
        ambition.updated_at = Set(Utc::now().into());
        ambition.update(db).await
    }

    pub async fn unarchive(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<ambition::Model, DbErr> {
        let mut ambition: ambition::ActiveModel =
            AmbitionQuery::find_by_id_and_user_id(db, ambition_id, user_id)
                .await?
                .into();
        ambition.archived = Set(false);
        ambition.updated_at = Set(Utc::now().into());
        ambition.update(db).await
    }

    pub async fn bulk_update_ordering(
        db: &DbConn,
        user_id: uuid::Uuid,
        ordering: Vec<uuid::Uuid>,
    ) -> Result<(), DbErr> {
        let ambitions = ambition::Entity::find()
            .filter(ambition::Column::UserId.eq(user_id))
            .filter(ambition::Column::Id.is_in(ordering.clone()))
            .all(db)
            .await?;
        for ambition in ambitions {
            let order = &ordering.iter().position(|id| id == &ambition.id);
            if let Some(order) = order {
                let mut ambition = ambition.into_active_model();
                ambition.ordering = Set(Some((order + 1) as i32));
                ambition.update(db).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ::types::CustomDbErr;
    use common::{
        db::init_db,
        factory::{self, *},
        settings::get_test_settings,
    };

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let name = "Test AmbitionMutation::create_with_tag".to_string();
        let description = Some("Dummy description".to_string());

        let form_data = NewAmbition {
            name: name.clone(),
            description: description.clone(),
            user_id: user.id,
        };

        let res = AmbitionMutation::create_with_tag(&db, form_data)
            .await
            .unwrap();
        assert_eq!(res.name, name);
        assert_eq!(res.description, description);
        assert_eq!(res.archived, false);
        assert_eq!(res.user_id, user.id);

        let ambition_in_db = ambition::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db, res);

        let tag_in_db = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::AmbitionId.eq(Some(res.id)))
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
        let ambition = factory::ambition(user.id).insert(&db).await?;

        let new_name = "Test AmbitionMutation::update_after".to_string();
        let new_description = Some("After update.".to_string());

        let res = AmbitionMutation::update(
            &db,
            ambition.id,
            user.id,
            new_name.clone(),
            new_description.clone(),
        )
        .await?;
        assert_eq!(res.id, ambition.id);
        assert_eq!(res.name, new_name);
        assert_eq!(res.description, new_description);
        assert_eq!(res.archived, ambition.archived);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, ambition.created_at);
        assert!(res.updated_at > ambition.updated_at);

        let ambition_in_db = ambition::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;

        let new_name = "Test AmbitionMutation::update_after".to_string();
        let new_description = Some("After update.".to_string());

        let error = AmbitionMutation::update(
            &db,
            ambition.id,
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
        let (ambition, tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

        AmbitionMutation::delete(&db, ambition.id, user.id).await?;

        let ambition_in_db = ambition::Entity::find_by_id(ambition.id).one(&db).await?;
        assert!(ambition_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;

        let error = AmbitionMutation::delete(&db, ambition.id, uuid::Uuid::now_v7())
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
        let ambition = factory::ambition(user.id).insert(&db).await?;

        let res = AmbitionMutation::archive(&db, ambition.id, user.id).await?;
        assert_eq!(res.id, ambition.id);
        assert_eq!(res.name, ambition.name.clone());
        assert_eq!(res.description, ambition.description.clone());
        assert_eq!(res.archived, true);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, ambition.created_at);
        assert!(res.updated_at > ambition.updated_at);

        let ambition_in_db = ambition::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn archive_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;

        let error = AmbitionMutation::archive(&db, ambition.id, uuid::Uuid::now_v7())
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
        let ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = AmbitionMutation::unarchive(&db, ambition.id, user.id).await?;
        assert_eq!(res.id, ambition.id);
        assert_eq!(res.name, ambition.name.clone());
        assert_eq!(res.description, ambition.description.clone());
        assert_eq!(res.archived, false);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, ambition.created_at);
        assert!(res.updated_at > ambition.updated_at);

        let ambition_in_db = ambition::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn unarchive_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let error = AmbitionMutation::unarchive(&db, ambition.id, uuid::Uuid::now_v7())
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
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let ambition_1 = factory::ambition(user.id).insert(&db).await?;
        let ambition_2 = factory::ambition(user.id).insert(&db).await?;

        let ordering = vec![ambition_0.id, ambition_1.id];

        AmbitionMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let ambition_in_db_0 = ambition::Entity::find_by_id(ambition_0.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db_0.ordering, Some(1));

        let ambition_in_db_1 = ambition::Entity::find_by_id(ambition_1.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db_1.ordering, Some(2));

        let ambition_in_db_2 = ambition::Entity::find_by_id(ambition_2.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db_2.ordering, None);

        Ok(())
    }

    #[actix_web::test]
    async fn bulk_update_ordering_no_modification_on_different_users_records() -> Result<(), DbErr>
    {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let another_users_ambition = factory::ambition(another_user.id).insert(&db).await?;

        let ordering = vec![another_users_ambition.id];

        AmbitionMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let another_users_ambition_in_db = ambition::Entity::find_by_id(another_users_ambition.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(another_users_ambition_in_db.ordering, None);

        Ok(())
    }
}
