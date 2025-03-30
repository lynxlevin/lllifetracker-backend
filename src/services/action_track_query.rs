use ::types::{ActionTrackVisible, ActionTrackWithAction, CustomDbErr};
use chrono::{DateTime, FixedOffset};
use entities::{action, action_track};
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, QueryOrder, QuerySelect};

pub struct ActionTrackQueryFilters {
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
    pub show_active: bool,
    pub show_inactive: bool,
}

pub struct ActionTrackQuery;

impl ActionTrackQuery {
    pub fn get_default_filters() -> ActionTrackQueryFilters {
        ActionTrackQueryFilters {
            started_at_gte: None,
            started_at_lte: None,
            show_active: true,
            show_inactive: true,
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
        let query = if filters.show_active {
            query
        } else {
            query.filter(action_track::Column::EndedAt.is_not_null())
        };
        let query = if filters.show_inactive {
            query
        } else {
            query.filter(action_track::Column::EndedAt.is_null())
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
        started_at_gte: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> Result<Vec<ActionTrackWithAction>, DbErr> {
        let query = match active_only {
            true => action_track::Entity::find().filter(action_track::Column::EndedAt.is_null()),
            false => action_track::Entity::find(),
        };
        let query = match started_at_gte {
            Some(started_at_gte) => query.filter(action_track::Column::StartedAt.gte(started_at_gte)),
            None => query
        };
        query
            .filter(action_track::Column::UserId.eq(user_id))
            .column_as(action::Column::Name, "action_name")
            .column_as(action::Column::Color, "action_color")
            .join(LeftJoin, action_track::Relation::Action.def())
            .order_by_desc(action_track::Column::StartedAt)
            .into_model::<ActionTrackWithAction>()
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
        filters.show_active = false;

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

        let res = ActionTrackQuery::find_all_by_user_id(&db, user.id, false, None).await?;

        let expected = vec![
            ActionTrackWithAction {
                id: action_track_1.id,
                action_id: Some(action.id),
                action_name: Some(action.name),
                action_color: Some(action.color),
                started_at: action_track_1.started_at,
                ended_at: action_track_1.ended_at,
                duration: action_track_1.duration,
            },
            ActionTrackWithAction {
                id: action_track_0.id,
                action_id: None,
                action_name: None,
                action_color: None,
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

        let res = ActionTrackQuery::find_all_by_user_id(&db, user.id, true, None).await?;

        let expected = vec![ActionTrackWithAction {
            id: active_action_track.id,
            action_id: Some(action.id),
            action_name: Some(action.name),
            action_color: Some(action.color),
            started_at: active_action_track.started_at,
            ended_at: active_action_track.ended_at,
            duration: active_action_track.duration,
        }];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_by_user_id_started_at_gte() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let started_at_gte: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-03-27T00:00:00Z").unwrap();
        let action_track = factory::action_track(user.id)
            .started_at(started_at_gte)
            .duration(Some(120))
            .insert(&db)
            .await?;
        let _old_action_track = factory::action_track(user.id)
            .started_at(started_at_gte - Duration::seconds(1))
            .duration(Some(120))
            .insert(&db)
            .await?;

        let res = ActionTrackQuery::find_all_by_user_id(&db, user.id, false, Some(started_at_gte)).await?;

        let expected = vec![ActionTrackWithAction {
            id: action_track.id,
            action_id: None,
            action_name: None,
            action_color: None,
            started_at: action_track.started_at,
            ended_at: action_track.ended_at,
            duration: action_track.duration,
        }];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }
}
