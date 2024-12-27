use crate::entities::{action, action_track};
use crate::types::{ActionTrackWithActionName, CustomDbErr};
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, QueryOrder, QuerySelect};

pub struct ActionTrackQuery;

impl ActionTrackQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
        active_only: bool,
    ) -> Result<Vec<ActionTrackWithActionName>, DbErr> {
        let query = match active_only {
            true => action_track::Entity::find().filter(action_track::Column::EndedAt.is_null()),
            false => action_track::Entity::find(),
        };
        query
            .filter(action_track::Column::UserId.eq(user_id))
            .column_as(action::Column::Name, "action_name")
            .join(LeftJoin, action_track::Relation::Action.def())
            .order_by_desc(action_track::Column::StartedAt)
            .into_model::<ActionTrackWithActionName>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        action_track_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<action_track::Model, DbErr> {
        action_track::Entity::find_by_id(action_track_id)
            .filter(action_track::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{self, factory};

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track_0 =
            test_utils::seed::create_action_track(&db, Some(120), None, user.id).await?;
        let action_track_1 =
            test_utils::seed::create_action_track(&db, Some(180), Some(action.id), user.id).await?;

        let res = ActionTrackQuery::find_all_by_user_id(&db, user.id, false).await?;

        let expected = vec![
            ActionTrackWithActionName {
                id: action_track_0.id,
                action_id: None,
                action_name: None,
                started_at: action_track_0.started_at,
                ended_at: action_track_0.ended_at,
                duration: action_track_0.duration,
            },
            ActionTrackWithActionName {
                id: action_track_1.id,
                action_id: Some(action.id),
                action_name: Some(action.name),
                started_at: action_track_1.started_at,
                ended_at: action_track_1.ended_at,
                duration: action_track_1.duration,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_by_user_id_active_only() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let _inactive_action_track =
            test_utils::seed::create_action_track(&db, Some(120), None, user.id).await?;
        let active_action_track =
            test_utils::seed::create_action_track(&db, None, Some(action.id), user.id).await?;

        let res = ActionTrackQuery::find_all_by_user_id(&db, user.id, true).await?;

        let expected = vec![ActionTrackWithActionName {
            id: active_action_track.id,
            action_id: Some(action.id),
            action_name: Some(action.name),
            started_at: active_action_track.started_at,
            ended_at: active_action_track.ended_at,
            duration: active_action_track.duration,
        }];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }
}
