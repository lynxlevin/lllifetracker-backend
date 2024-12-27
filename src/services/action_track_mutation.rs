use crate::entities::action_track;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewActionTrack {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub action_id: Option<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

pub struct ActionTrackMutation;

impl ActionTrackMutation {
    pub async fn create(
        db: &DbConn,
        form_data: NewActionTrack,
    ) -> Result<action_track::Model, DbErr> {
        action_track::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set(form_data.user_id),
            action_id: Set(form_data.action_id),
            started_at: Set(form_data.started_at),
            ended_at: NotSet,
            duration: NotSet,
        }
        .insert(db)
        .await
    }

    // pub async fn update(
    //     db: &DbConn,
    //     action_id: uuid::Uuid,
    //     user_id: uuid::Uuid,
    //     name: String,
    //     description: Option<String>,
    // ) -> Result<action::Model, DbErr> {
    //     let mut action: action::ActiveModel =
    //         ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
    //             .await?
    //             .into();
    //     action.name = Set(name);
    //     action.description = Set(description);
    //     action.updated_at = Set(Utc::now().into());
    //     action.update(db).await
    // }

    // pub async fn delete(
    //     db: &DbConn,
    //     action_id: uuid::Uuid,
    //     user_id: uuid::Uuid,
    // ) -> Result<(), DbErr> {
    //     ActionQuery::find_by_id_and_user_id(db, action_id, user_id)
    //         .await?
    //         .delete(db)
    //         .await?;
    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sea_orm::DbErr;

    use crate::test_utils;
    use crate::types::CustomDbErr;

    use super::*;

    #[actix_web::test]
    async fn create() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action =
            test_utils::seed::create_action(&db, "action".to_string(), None, user.id).await?;

        let form_data = NewActionTrack {
            started_at: Utc::now().into(),
            action_id: Some(action.id),
            user_id: user.id,
        };

        let returned_action_track = ActionTrackMutation::create(&db, form_data.clone())
            .await
            .unwrap();
        assert_eq!(returned_action_track.user_id, user.id);
        assert_eq!(returned_action_track.action_id, Some(action.id));
        assert_eq!(returned_action_track.started_at, form_data.started_at);
        assert_eq!(returned_action_track.ended_at, None);
        assert_eq!(returned_action_track.duration, None);

        let created_action_track = action_track::Entity::find_by_id(returned_action_track.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_action_track.user_id, user.id);
        assert_eq!(created_action_track.action_id, Some(action.id));
        assert_eq!(created_action_track.started_at, form_data.started_at);
        assert_eq!(created_action_track.ended_at, None);
        assert_eq!(created_action_track.duration, None);

        Ok(())
    }

    // #[actix_web::test]
    // async fn update() -> Result<(), DbErr> {
    //     let db = test_utils::init_db().await?;
    //     let user = test_utils::seed::create_active_user(&db).await?;
    //     let (action, _) = test_utils::seed::create_action_and_tag(
    //         &db,
    //         "action_before_update".to_string(),
    //         None,
    //         user.id,
    //     )
    //     .await?;
    //     let new_name = "action_after_update".to_string();
    //     let new_description = "Action after update.".to_string();

    //     let returned_action = ActionMutation::update(
    //         &db,
    //         action.id,
    //         user.id,
    //         new_name.clone(),
    //         Some(new_description.clone()),
    //     )
    //     .await?;
    //     assert_eq!(returned_action.id, action.id);
    //     assert_eq!(returned_action.name, new_name.clone());
    //     assert_eq!(returned_action.description, Some(new_description.clone()));
    //     assert_eq!(returned_action.archived, action.archived);
    //     assert_eq!(returned_action.user_id, user.id);
    //     assert_eq!(returned_action.created_at, action.created_at);
    //     assert!(returned_action.updated_at > action.updated_at);

    //     let updated_action = action::Entity::find_by_id(action.id)
    //         .one(&db)
    //         .await?
    //         .unwrap();
    //     assert_eq!(updated_action.id, action.id);
    //     assert_eq!(updated_action.name, new_name.clone());
    //     assert_eq!(updated_action.description, Some(new_description.clone()));
    //     assert_eq!(updated_action.archived, action.archived);
    //     assert_eq!(updated_action.user_id, user.id);
    //     assert_eq!(updated_action.created_at, action.created_at);
    //     assert_eq!(updated_action.updated_at, returned_action.updated_at);

    //     Ok(())
    // }

    // #[actix_web::test]
    // async fn update_unauthorized() -> Result<(), DbErr> {
    //     let db = test_utils::init_db().await?;
    //     let user = test_utils::seed::create_active_user(&db).await?;
    //     let (action, _) = test_utils::seed::create_action_and_tag(
    //         &db,
    //         "action_before_update_unauthorized".to_string(),
    //         None,
    //         user.id,
    //     )
    //     .await?;
    //     let new_name = "action_after_update_unauthorized".to_string();

    //     let error =
    //         ActionMutation::update(&db, action.id, uuid::Uuid::new_v4(), new_name.clone(), None)
    //             .await
    //             .unwrap_err();
    //     assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

    //     Ok(())
    // }

    // #[actix_web::test]
    // async fn delete() -> Result<(), DbErr> {
    //     let db = test_utils::init_db().await?;
    //     let user = test_utils::seed::create_active_user(&db).await?;
    //     let (action, tag) = test_utils::seed::create_action_and_tag(
    //         &db,
    //         "action_for_delete".to_string(),
    //         None,
    //         user.id,
    //     )
    //     .await?;

    //     ActionMutation::delete(&db, action.id, user.id).await?;

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
    //         None,
    //         user.id,
    //     )
    //     .await?;

    //     let error = ActionMutation::delete(&db, action.id, uuid::Uuid::new_v4())
    //         .await
    //         .unwrap_err();
    //     assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

    //     Ok(())
    // }
}
