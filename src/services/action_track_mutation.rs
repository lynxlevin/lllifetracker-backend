use crate::entities::action_track;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};

use super::action_track_query::ActionTrackQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewActionTrack {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub action_id: Option<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct UpdateActionTrack {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
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

    pub async fn update(
        db: &DbConn,
        action_track_id: uuid::Uuid,
        form_data: UpdateActionTrack,
    ) -> Result<action_track::Model, DbErr> {
        let mut action_track: action_track::ActiveModel =
            ActionTrackQuery::find_by_id_and_user_id(db, action_track_id, form_data.user_id)
                .await?
                .into();
        action_track.action_id = Set(form_data.action_id);
        action_track.started_at = Set(form_data.started_at);
        action_track.ended_at = Set(form_data.ended_at);
        match form_data.ended_at {
            Some(ended_at) => action_track.duration = Set(Some((ended_at - form_data.started_at).num_seconds())),
            None => action_track.duration = Set(None),
        }
        action_track.update(db).await
    }

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

    #[actix_web::test]
    async fn update() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action =
            test_utils::seed::create_action(&db, "action".to_string(), None, user.id).await?;
        let action_track = test_utils::seed::create_action_track(&db, None, None, user.id).await?;
        let ended_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let duration = 180;
        let started_at = ended_at - chrono::TimeDelta::seconds(duration.into());

        let returned_action = ActionTrackMutation::update(
            &db,
            action_track.id,
            UpdateActionTrack {
                user_id: user.id,
                action_id: Some(action.id),
                started_at: started_at,
                ended_at: Some(ended_at),
            }
        )
        .await?;
        assert_eq!(returned_action.id, action_track.id);
        assert_eq!(returned_action.action_id, Some(action.id));
        assert_eq!(returned_action.user_id, user.id);
        assert_eq!(returned_action.started_at, started_at);
        assert_eq!(returned_action.ended_at, Some(ended_at));
        assert_eq!(returned_action.duration, Some(duration));

        let updated_action = action_track::Entity::find_by_id(action_track.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_action.action_id, Some(action.id));
        assert_eq!(updated_action.user_id, user.id);
        assert_eq!(updated_action.started_at, started_at);
        assert_eq!(updated_action.ended_at, Some(ended_at));
        assert_eq!(updated_action.duration, Some(duration));

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action_track = test_utils::seed::create_action_track(&db, None, None, user.id).await?;

        let error = ActionTrackMutation::update(
            &db,
            action_track.id,
            UpdateActionTrack {
                user_id: uuid::Uuid::new_v4(),
                action_id: None,
                started_at: Utc::now().into(),
                ended_at: None,
            }
        )
        .await
        .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

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
