use chrono::Utc;
use entities::{action, sea_orm_active_enums::ActionTrackType, tag};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, Set, TransactionError, TransactionTrait,
};

use super::action_query::ActionQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewAction {
    pub name: String,
    pub description: Option<String>,
    pub track_type: ActionTrackType,
    pub user_id: uuid::Uuid,
}

pub struct ActionMutation;

impl ActionMutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewAction,
    ) -> Result<action::Model, TransactionError<DbErr>> {
        db.transaction::<_, action::Model, DbErr>(|txn| {
            Box::pin(async move {
                let action_id = uuid::Uuid::now_v7();
                let created_action = action::ActiveModel {
                    id: Set(action_id),
                    user_id: Set(form_data.user_id),
                    name: Set(form_data.name.to_owned()),
                    description: Set(form_data.description.to_owned()),
                    track_type: Set(form_data.track_type),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::now_v7()),
                    user_id: Set(form_data.user_id),
                    action_id: Set(Some(action_id)),
                    ..Default::default()
                }
                .insert(txn)
                .await?;

                Ok(created_action)
            })
        })
        .await
    }

    pub async fn update(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
        name: String,
        description: Option<String>,
        trackable: Option<bool>,
        color: Option<String>,
    ) -> Result<action::Model, DbErr> {
        let mut action: action::ActiveModel =
            ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
                .await?
                .into();
        action.name = Set(name);
        action.description = Set(description);
        if let Some(trackable) = trackable {
            action.trackable = Set(trackable);
        }
        if let Some(color) = color {
            action.color = Set(color);
        }
        action.updated_at = Set(Utc::now().into());
        action.update(db).await
    }

    pub async fn convert_track_type(
        db: &DbConn,
        action: action::Model,
        track_type: ActionTrackType,
    ) -> Result<action::Model, DbErr> {
        let mut action = action.into_active_model();
        action.track_type = Set(track_type);
        action.updated_at = Set(Utc::now().into());
        action.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn archive(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<action::Model, DbErr> {
        let mut action: action::ActiveModel =
            ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
                .await?
                .into();
        action.archived = Set(true);
        action.updated_at = Set(Utc::now().into());
        action.update(db).await
    }

    pub async fn unarchive(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<action::Model, DbErr> {
        let mut action: action::ActiveModel =
            ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
                .await?
                .into();
        action.archived = Set(false);
        action.updated_at = Set(Utc::now().into());
        action.update(db).await
    }

    pub async fn bulk_update_ordering(
        db: &DbConn,
        user_id: uuid::Uuid,
        ordering: Vec<uuid::Uuid>,
    ) -> Result<(), DbErr> {
        let actions = action::Entity::find()
            .filter(action::Column::UserId.eq(user_id))
            .filter(action::Column::Id.is_in(ordering.clone()))
            .all(db)
            .await?;
        for action in actions {
            let order = &ordering.iter().position(|id| id == &action.id);
            if let Some(order) = order {
                let mut action = action.into_active_model();
                action.ordering = Set(Some((order + 1) as i32));
                action.update(db).await?;
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
    use sea_orm::DbErr;

    use ::types::CustomDbErr;
    use entities::tag;

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action_name = "create_with_tag".to_string();
        let action_description = "Create with Tag.".to_string();

        let form_data = NewAction {
            name: action_name.clone(),
            description: Some(action_description.clone()),
            track_type: ActionTrackType::Count,
            user_id: user.id,
        };

        let res = ActionMutation::create_with_tag(&db, form_data)
            .await
            .unwrap();
        assert_eq!(res.name, action_name.clone());
        assert_eq!(res.description, Some(action_description.clone()));
        assert_eq!(res.track_type, ActionTrackType::Count);
        assert_eq!(res.archived, false);
        assert_eq!(res.user_id, user.id);

        let action_in_db = action::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(action_in_db, res);

        let tag_in_db = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::DesiredStateId.is_null())
            .filter(tag::Column::ActionId.eq(res.id))
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
        let action = factory::action(user.id).insert(&db).await?;

        let new_name = "action_after_update".to_string();
        let new_description = "Action after update.".to_string();
        let new_trackable = false;
        let new_color = "#ffffff".to_string();

        let res = ActionMutation::update(
            &db,
            action.id,
            user.id,
            new_name.clone(),
            Some(new_description.clone()),
            Some(new_trackable),
            Some(new_color.clone()),
        )
        .await?;
        assert_eq!(res.id, action.id);
        assert_eq!(res.name, new_name.clone());
        assert_eq!(res.description, Some(new_description.clone()));
        assert_eq!(res.archived, action.archived);
        assert_eq!(res.trackable, new_trackable);
        assert_eq!(res.color, new_color.clone());
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.created_at, action.created_at);
        assert!(res.updated_at > action.updated_at);

        let action_in_db = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(action_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let new_name = "action_after_update_unauthorized".to_string();

        let error = ActionMutation::update(
            &db,
            action.id,
            uuid::Uuid::now_v7(),
            new_name.clone(),
            None,
            None,
            None,
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
        let (action, tag) = factory::action(user.id).insert_with_tag(&db).await?;

        ActionMutation::delete(&db, action.id, user.id).await?;

        let action_in_db = action::Entity::find_by_id(action.id).one(&db).await?;
        assert!(action_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let error = ActionMutation::delete(&db, action.id, uuid::Uuid::now_v7())
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
        let action = factory::action(user.id).insert(&db).await?;

        ActionMutation::archive(&db, action.id, user.id).await?;

        let action_in_db = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert!(action_in_db.archived);

        Ok(())
    }

    #[actix_web::test]
    async fn archive_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let error = ActionMutation::archive(&db, action.id, uuid::Uuid::now_v7())
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
        let action = factory::action(user.id).archived(true).insert(&db).await?;

        ActionMutation::unarchive(&db, action.id, user.id).await?;

        let action_in_db = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert!(!action_in_db.archived);

        Ok(())
    }

    #[actix_web::test]
    async fn unarchive_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).archived(true).insert(&db).await?;

        let error = ActionMutation::unarchive(&db, action.id, uuid::Uuid::now_v7())
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
        let action_0 = factory::action(user.id).insert(&db).await?;
        let action_1 = factory::action(user.id).insert(&db).await?;
        let action_2 = factory::action(user.id).insert(&db).await?;

        let ordering = vec![action_0.id, action_1.id];

        ActionMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let action_in_db_0 = action::Entity::find_by_id(action_0.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(action_in_db_0.ordering, Some(1));

        let action_in_db_1 = action::Entity::find_by_id(action_1.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(action_in_db_1.ordering, Some(2));

        let action_in_db_2 = action::Entity::find_by_id(action_2.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(action_in_db_2.ordering, None);

        Ok(())
    }

    #[actix_web::test]
    async fn bulk_update_ordering_no_modification_on_different_users_records() -> Result<(), DbErr>
    {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let another_users_action = factory::action(another_user.id).insert(&db).await?;

        let ordering = vec![another_users_action.id];

        ActionMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let another_users_action_in_db = action::Entity::find_by_id(another_users_action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(another_users_action_in_db.ordering, None);

        Ok(())
    }
}
