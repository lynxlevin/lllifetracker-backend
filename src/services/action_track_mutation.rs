use chrono::SubsecRound;
use entities::action_track;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, ModelTrait, Set};
use types::CustomDbErr;

use super::action_track_query::ActionTrackQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewActionTrack {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub action_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct UpdateActionTrack {
    pub started_at: chrono::DateTime<chrono::FixedOffset>,
    pub ended_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub action_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

pub struct ActionTrackMutation;

impl ActionTrackMutation {
    pub async fn create(
        db: &DbConn,
        form_data: NewActionTrack,
    ) -> Result<action_track::Model, DbErr> {
        let started_at = form_data.started_at.trunc_subsecs(0);
        action_track::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(form_data.user_id),
            action_id: Set(form_data.action_id),
            started_at: Set(started_at),
            ended_at: match form_data.ended_at {
                Some(ended_at) => Set(Some(ended_at.trunc_subsecs(0))),
                None => NotSet,
            },
            duration: match form_data.ended_at {
                Some(ended_at) => Set(Some((ended_at.trunc_subsecs(0) - started_at).num_seconds())),
                None => NotSet,
            },
        }
        .insert(db)
        .await
        .or(Err(DbErr::Custom(CustomDbErr::Duplicate.to_string())))
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
        action_track.started_at = Set(form_data.started_at.trunc_subsecs(0));
        match form_data.ended_at {
            Some(ended_at) => {
                action_track.ended_at = Set(Some(ended_at.trunc_subsecs(0)));
                action_track.duration = Set(Some((ended_at - form_data.started_at).num_seconds()))
            }
            None => {
                action_track.ended_at = Set(None);
                action_track.duration = Set(None)
            }
        }
        action_track
            .update(db)
            .await
            .or(Err(DbErr::Custom(CustomDbErr::Duplicate.to_string())))
    }

    pub async fn delete(
        db: &DbConn,
        action_track_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        ActionTrackQuery::find_by_id_and_user_id(db, action_track_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, Utc};
    use sea_orm::EntityTrait;

    use ::types::CustomDbErr;
    use common::{
        db::init_db,
        factory::{self, *},
        settings::get_test_settings,
    };
    use entities::action;

    use super::*;

    #[actix_web::test]
    async fn create() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let form_data = NewActionTrack {
            started_at: Utc::now().into(),
            ended_at: None,
            action_id: action.id,
            user_id: user.id,
        };

        let res = ActionTrackMutation::create(&db, form_data.clone())
            .await
            .unwrap();
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.action_id, action.id);
        assert_eq!(res.started_at, form_data.started_at.trunc_subsecs(0));
        assert_eq!(res.ended_at, None);
        assert_eq!(res.duration, None);

        let action_track_in_db = action_track::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(action_track_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn create_duplicate_creation() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let existing_action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;

        let form_data = NewActionTrack {
            started_at: existing_action_track.started_at,
            ended_at: None,
            action_id: action.id,
            user_id: user.id,
        };

        let error = ActionTrackMutation::create(&db, form_data)
            .await
            .unwrap_err();

        assert_eq!(error, DbErr::Custom(CustomDbErr::Duplicate.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn update() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;
        let ended_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let duration = 180;
        let started_at = ended_at - chrono::TimeDelta::seconds(duration.into());

        let res = ActionTrackMutation::update(
            &db,
            action_track.id,
            UpdateActionTrack {
                user_id: user.id,
                action_id: action.id,
                started_at: started_at,
                ended_at: Some(ended_at),
            },
        )
        .await?;
        assert_eq!(res.id, action_track.id);
        assert_eq!(res.action_id, action.id);
        assert_eq!(res.user_id, user.id);
        assert_eq!(res.started_at, started_at.trunc_subsecs(0));
        assert_eq!(res.ended_at, Some(ended_at.trunc_subsecs(0)));
        assert_eq!(res.duration, Some(duration));

        let action_track_in_db = action_track::Entity::find_by_id(action_track.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(action_track_in_db, res);

        Ok(())
    }

    #[actix_web::test]
    async fn update_conflicting_update() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;
        let existing_action_track = factory::action_track(user.id)
            .started_at(action_track.started_at + TimeDelta::seconds(1))
            .action_id(action.id)
            .insert(&db)
            .await?;

        let error = ActionTrackMutation::update(
            &db,
            action_track.id,
            UpdateActionTrack {
                user_id: user.id,
                action_id: action.id,
                started_at: existing_action_track.started_at,
                ended_at: None,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error, DbErr::Custom(CustomDbErr::Duplicate.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;

        let error = ActionTrackMutation::update(
            &db,
            action_track.id,
            UpdateActionTrack {
                user_id: uuid::Uuid::now_v7(),
                action_id: action.id,
                started_at: Utc::now().into(),
                ended_at: None,
            },
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
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;

        ActionTrackMutation::delete(&db, action_track.id, user.id).await?;

        let action_track_in_db = action_track::Entity::find_by_id(action_track.id)
            .one(&db)
            .await?;
        assert!(action_track_in_db.is_none());

        let action_in_db = action::Entity::find_by_id(action.id).one(&db).await?;
        assert!(action_in_db.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track = factory::action_track(user.id)
            .action_id(action.id)
            .insert(&db)
            .await?;

        let error = ActionTrackMutation::delete(&db, action_track.id, uuid::Uuid::now_v7())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }
}
