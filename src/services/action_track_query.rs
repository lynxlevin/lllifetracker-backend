use entities::{action, action_track};
use ::types::{ActionTrackVisible, ActionTrackWithActionName, CustomDbErr};
use chrono::{DateTime, FixedOffset};
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, QueryOrder, QuerySelect};

pub struct ActionTrackQueryFilters {
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
    pub inactive_only: bool,
    pub with_action_id: bool,
}

pub struct ActionTrackQuery;

impl ActionTrackQuery {
    pub fn get_default_filters() -> ActionTrackQueryFilters {
        ActionTrackQueryFilters {
            started_at_gte: None,
            started_at_lte: None,
            inactive_only: false,
            with_action_id: true,
        }
    }

    pub async fn find_by_user_id_with_filters(
        db: &DbConn,
        user_id: uuid::Uuid,
        filters: ActionTrackQueryFilters,
    ) -> Result<Vec<ActionTrackVisible>, DbErr> {
        let query = action_track::Entity::find();
        let query = if let Some(started_at_gte) = filters.started_at_gte {
            query.filter(action_track::Column::StartedAt.gte(started_at_gte))
        } else {
            query
        };
        let query = if let Some(started_at_lte) = filters.started_at_lte {
            query.filter(action_track::Column::StartedAt.lte(started_at_lte))
        } else {
            query
        };
        let query = if filters.inactive_only {
            query.filter(action_track::Column::EndedAt.is_not_null())
        } else {
            query
        };
        let query = if filters.with_action_id {
            query.filter(action_track::Column::ActionId.is_not_null())
        } else {
            query.filter(action_track::Column::ActionId.is_null())
        };
        query
            .filter(action_track::Column::UserId.eq(user_id))
            .order_by_asc(action_track::Column::ActionId)
            .order_by_desc(action_track::Column::StartedAt)
            .into_model::<ActionTrackVisible>()
            .all(db)
            .await
    }

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
    use chrono::Duration;

    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn find_by_user_id_with_filters() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let query_started_at_gte: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-01-27T00:00:00Z").unwrap();
        let query_started_at_lte: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-01-27T23:59:59Z").unwrap();
        let action = factory::action(user.id).insert(&db).await?;
        let _action_track_0 = factory::action_track(user.id)
            .started_at(query_started_at_gte - Duration::seconds(1))
            .duration(Some(120))
            .action_id(Some(action.id))
            .insert(&db)
            .await?;
        let action_track_1 = factory::action_track(user.id)
            .started_at(query_started_at_gte)
            .duration(Some(180))
            .action_id(Some(action.id))
            .insert(&db)
            .await?;
        let action_track_2 = factory::action_track(user.id)
            .started_at(query_started_at_lte)
            .duration(Some(350))
            .action_id(Some(action.id))
            .insert(&db)
            .await?;
        let _action_track_3 = factory::action_track(user.id)
            .started_at(query_started_at_lte + Duration::seconds(1))
            .duration(Some(550))
            .action_id(Some(action.id))
            .insert(&db)
            .await?;

        let mut filters = ActionTrackQuery::get_default_filters();
        filters.started_at_gte = Some(query_started_at_gte);
        filters.started_at_lte = Some(query_started_at_lte);
        filters.inactive_only = true;

        let res = ActionTrackQuery::find_by_user_id_with_filters(&db, user.id, filters).await?;
        dbg!(&res);

        let expected = vec![
            ActionTrackVisible {
                id: action_track_2.id,
                action_id: Some(action.id),
                started_at: action_track_2.started_at,
                ended_at: action_track_2.ended_at,
                duration: action_track_2.duration,
            },
            ActionTrackVisible {
                id: action_track_1.id,
                action_id: Some(action.id),
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
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let action_track_0 = factory::action_track(user.id)
            .duration(Some(120))
            .insert(&db)
            .await?;
        let action_track_1 = factory::action_track(user.id)
            .duration(Some(180))
            .action_id(Some(action.id))
            .insert(&db)
            .await?;

        let res = ActionTrackQuery::find_all_by_user_id(&db, user.id, false).await?;

        let expected = vec![
            ActionTrackWithActionName {
                id: action_track_1.id,
                action_id: Some(action.id),
                action_name: Some(action.name),
                started_at: action_track_1.started_at,
                ended_at: action_track_1.ended_at,
                duration: action_track_1.duration,
            },
            ActionTrackWithActionName {
                id: action_track_0.id,
                action_id: None,
                action_name: None,
                started_at: action_track_0.started_at,
                ended_at: action_track_0.ended_at,
                duration: action_track_0.duration,
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
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let _inactive_action_track = factory::action_track(user.id)
            .duration(Some(120))
            .insert(&db)
            .await?;
        let active_action_track = factory::action_track(user.id)
            .action_id(Some(action.id))
            .insert(&db)
            .await?;

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
