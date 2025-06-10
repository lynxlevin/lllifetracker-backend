use chrono::Utc;
use entities::{desired_state, mindset, tag};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, Set, TransactionError, TransactionTrait,
};
use types::{CustomDbErr, DesiredStateConvertToType};

use super::desired_state_query::DesiredStateQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewDesiredState {
    pub name: String,
    pub description: Option<String>,
    pub user_id: uuid::Uuid,
}

pub struct DesiredStateMutation;

impl DesiredStateMutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewDesiredState,
    ) -> Result<desired_state::Model, TransactionError<DbErr>> {
        db.transaction::<_, desired_state::Model, DbErr>(|txn| {
            Box::pin(async move {
                let desired_state_id = uuid::Uuid::now_v7();
                let created_desired_state = desired_state::ActiveModel {
                    id: Set(desired_state_id),
                    user_id: Set(form_data.user_id),
                    name: Set(form_data.name.to_owned()),
                    description: Set(form_data.description.to_owned()),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::now_v7()),
                    user_id: Set(form_data.user_id),
                    desired_state_id: Set(Some(desired_state_id)),
                    ..Default::default()
                }
                .insert(txn)
                .await?;

                Ok(created_desired_state)
            })
        })
        .await
    }

    pub async fn update(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<desired_state::Model, DbErr> {
        let mut desired_state: desired_state::ActiveModel =
            DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
                .await?
                .into();
        desired_state.name = Set(name);
        desired_state.description = Set(description);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn archive(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<desired_state::Model, DbErr> {
        let mut desired_state: desired_state::ActiveModel =
            DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
                .await?
                .into();
        desired_state.archived = Set(true);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(db).await
    }

    pub async fn unarchive(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<desired_state::Model, DbErr> {
        let mut desired_state: desired_state::ActiveModel =
            DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
                .await?
                .into();
        desired_state.archived = Set(false);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(db).await
    }

    // FIXME: Reduce query.
    pub async fn bulk_update_ordering(
        db: &DbConn,
        user_id: uuid::Uuid,
        ordering: Vec<uuid::Uuid>,
    ) -> Result<(), DbErr> {
        let desired_states = desired_state::Entity::find()
            .filter(desired_state::Column::UserId.eq(user_id))
            .filter(desired_state::Column::Id.is_in(ordering.clone()))
            .all(db)
            .await?;
        for desired_state in desired_states {
            let order = &ordering.iter().position(|id| id == &desired_state.id);
            if let Some(order) = order {
                let mut desired_state = desired_state.into_active_model();
                desired_state.ordering = Set(Some((order + 1) as i32));
                desired_state.update(db).await?;
            }
        }
        Ok(())
    }
    pub async fn convert(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
        convert_to: DesiredStateConvertToType,
    ) -> Result<mindset::Model, TransactionError<DbErr>> {
        let desired_state_result =
            DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id).await;
        db.transaction::<_, mindset::Model, DbErr>(|txn| {
            Box::pin(async move {
                let desired_state = desired_state_result?;
                let mut tag = tag::Entity::find()
                    .filter(tag::Column::UserId.eq(user_id))
                    .filter(tag::Column::DesiredStateId.eq(desired_state_id))
                    .one(txn)
                    .await?
                    .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))?
                    .into_active_model();
                let converted = match convert_to {
                    DesiredStateConvertToType::Mindset => {
                        let mindset = mindset::ActiveModel {
                            id: Set(uuid::Uuid::now_v7()),
                            user_id: Set(user_id),
                            name: Set(desired_state.name.clone()),
                            description: Set(desired_state.description.clone()),
                            ..Default::default()
                        }
                        .insert(txn)
                        .await?;
                        tag.mindset_id = Set(Some(mindset.id));
                        mindset
                    }
                };
                tag.desired_state_id = Set(None);
                tag.update(txn).await?;
                desired_state.delete(txn).await?;

                Ok(converted)
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use ::types::CustomDbErr;
    use common::factory::{self, *};
    use test_utils;
    use types::DesiredStateConvertToType;

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let name = "create_with_tag".to_string();
        let description = "Create with tag.".to_string();

        let form_data = NewDesiredState {
            name: name.clone(),
            description: Some(description.clone()),
            user_id: user.id,
        };
        let res = DesiredStateMutation::create_with_tag(&db, form_data)
            .await
            .unwrap();
        assert_eq!(res.name, name.clone());
        assert_eq!(res.description, Some(description.clone()));
        assert_eq!(res.archived, false);
        assert_eq!(res.user_id, user.id);

        let desired_state_in_db = desired_state::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db, res);

        let tag_in_db = tag::Entity::find()
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::DesiredStateId.eq(res.id))
            .filter(tag::Column::ActionId.is_null())
            .filter(tag::Column::UserId.eq(user.id))
            .one(&db)
            .await?;
        assert!(tag_in_db.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let new_name = "desired_state_after_update".to_string();
        let new_description = "DesiredState after update.".to_string();

        let res = DesiredStateMutation::update(
            &db,
            desired_state.id,
            user.id,
            new_name.clone(),
            Some(new_description.clone()),
        )
        .await?;
        assert_eq!(res.id, desired_state.id);
        assert_eq!(res.name, new_name.clone());
        assert_eq!(res.description, Some(new_description.clone()));
        assert_eq!(res.archived, desired_state.archived);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, desired_state.created_at);
        assert!(res.updated_at > desired_state.updated_at);

        let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let new_name = "desired_state_after_update_unauthorized".to_string();

        let error = DesiredStateMutation::update(
            &db,
            desired_state.id,
            uuid::Uuid::now_v7(),
            new_name.clone(),
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let (desired_state, tag) = factory::desired_state(user.id).insert_with_tag(&db).await?;

        DesiredStateMutation::delete(&db, desired_state.id, user.id).await?;

        let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?;
        assert!(desired_state_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let error = DesiredStateMutation::delete(&db, desired_state.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn archive() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let res = DesiredStateMutation::archive(&db, desired_state.id, user.id).await?;
        assert_eq!(res.id, desired_state.id);
        assert_eq!(res.name, desired_state.name.clone());
        assert_eq!(res.description, desired_state.description.clone());
        assert_eq!(res.archived, true);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, desired_state.created_at);
        assert!(res.updated_at > desired_state.updated_at);

        let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn archive_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let error = DesiredStateMutation::archive(&db, desired_state.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn unarchive() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = DesiredStateMutation::unarchive(&db, desired_state.id, user.id).await?;
        assert_eq!(res.id, desired_state.id);
        assert_eq!(res.name, desired_state.name.clone());
        assert_eq!(res.description, desired_state.description.clone());
        assert_eq!(res.archived, false);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, desired_state.created_at);
        assert!(res.updated_at > desired_state.updated_at);

        let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn unarchive_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let error = DesiredStateMutation::archive(&db, desired_state.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn bulk_update_ordering() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state_0 = factory::desired_state(user.id).insert(&db).await?;
        let desired_state_1 = factory::desired_state(user.id).insert(&db).await?;
        let desired_state_2 = factory::desired_state(user.id).insert(&db).await?;

        let ordering = vec![desired_state_0.id, desired_state_1.id];

        DesiredStateMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let desired_state_in_db_0 = desired_state::Entity::find_by_id(desired_state_0.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db_0.ordering, Some(1));

        let desired_state_in_db_1 = desired_state::Entity::find_by_id(desired_state_1.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db_1.ordering, Some(2));

        let desired_state_in_db_2 = desired_state::Entity::find_by_id(desired_state_2.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db_2.ordering, None);

        Ok(())
    }

    #[actix_web::test]
    async fn bulk_update_ordering_no_modification_on_different_users_records() -> Result<(), DbErr>
    {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let another_users_desired_state =
            factory::desired_state(another_user.id).insert(&db).await?;

        let ordering = vec![another_users_desired_state.id];

        DesiredStateMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let another_users_desired_state_in_db =
            desired_state::Entity::find_by_id(another_users_desired_state.id)
                .one(&db)
                .await?
                .unwrap();
        assert_eq!(another_users_desired_state_in_db.ordering, None);

        Ok(())
    }

    #[actix_web::test]
    async fn convert() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let (desired_state, desired_state_tag) =
            factory::desired_state(user.id).insert_with_tag(&db).await?;

        let res = DesiredStateMutation::convert(
            &db,
            desired_state.id,
            user.id,
            DesiredStateConvertToType::Mindset,
        )
        .await
        .unwrap();
        assert_ne!(res.id, desired_state.id);
        assert_eq!(res.name, desired_state.name);
        assert_eq!(res.description, desired_state.description);
        assert_eq!(res.archived, false);
        assert_eq!(res.user_id, user.id);
        assert_ne!(res.created_at, desired_state.created_at);
        assert_ne!(res.updated_at, desired_state.updated_at);

        let mindset_in_db = mindset::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(mindset_in_db, res);

        let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?;
        assert!(desired_state_in_db.is_none());

        dbg!(&desired_state_tag);
        let tag_in_db = tag::Entity::find_by_id(desired_state_tag.id)
            .one(&db)
            .await?
            .unwrap();
        assert!(tag_in_db.ambition_id.is_none());
        assert!(tag_in_db.desired_state_id.is_none());
        assert_eq!(tag_in_db.mindset_id, Some(res.id));
        assert!(tag_in_db.action_id.is_none());
        assert!(tag_in_db.name.is_none());
        assert_eq!(tag_in_db.user_id, user.id);
        assert_eq!(tag_in_db.created_at, desired_state_tag.created_at);

        Ok(())
    }

    #[actix_web::test]
    async fn convert_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let error = DesiredStateMutation::convert(
            &db,
            desired_state.id,
            uuid::Uuid::now_v7(),
            DesiredStateConvertToType::Mindset,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            TransactionError::Transaction(DbErr::Custom(CustomDbErr::NotFound.to_string()))
                .to_string()
        );

        Ok(())
    }
}
