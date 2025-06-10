use ::types::{ActionTrackVisible, CustomDbErr};
use chrono::{DateTime, FixedOffset};
use entities::{
    action,
    action_track::{self, Column},
};
use sea_orm::{
    ColumnTrait, DbConn, DbErr, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};

pub struct ActionTrackQueryFilters {
    pub started_at_gte: Option<DateTime<FixedOffset>>,
    pub started_at_lte: Option<DateTime<FixedOffset>>,
    pub show_active: bool,
    pub show_inactive: bool,
    pub for_daily_aggregation: bool,
}

pub struct ActionTrackQuery;

impl ActionTrackQuery {
    pub fn get_default_filters() -> ActionTrackQueryFilters {
        ActionTrackQueryFilters {
            started_at_gte: None,
            started_at_lte: None,
            show_active: true,
            show_inactive: true,
            for_daily_aggregation: false,
        }
    }

    pub async fn find_by_user_id_with_filters(
        db: &DbConn,
        user_id: uuid::Uuid,
        filters: ActionTrackQueryFilters,
    ) -> Result<Vec<ActionTrackVisible>, DbErr> {
        let query = action_track::Entity::find();
        let query = match filters.started_at_gte {
            Some(started_at_gte) => query.filter(Column::StartedAt.gte(started_at_gte)),
            None => query,
        };
        let query = match filters.started_at_lte {
            Some(started_at_lte) => query.filter(Column::StartedAt.lte(started_at_lte)),
            None => query,
        };
        let query = match filters.show_active {
            true => query,
            false => query.filter(Column::EndedAt.is_not_null()),
        };
        let query = match filters.show_inactive {
            true => query,
            false => query.filter(Column::EndedAt.is_null()),
        };

        let query = query.filter(Column::UserId.eq(user_id));
        let query = match filters.for_daily_aggregation {
            true => query.order_by_asc(action_track::Column::ActionId),
            false => query,
        };
        query
            .filter(action::Column::Archived.eq(false))
            .join(LeftJoin, action_track::Relation::Action.def())
            .order_by_desc(Column::StartedAt)
            .into_model::<ActionTrackVisible>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        action_track_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<action_track::Model, DbErr> {
        action_track::Entity::find_by_id(action_track_id)
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use sea_orm::ActiveModelTrait;

    use common::{
        db::init_db,
        factory::{self, *},
        settings::get_test_settings,
    };

    use super::*;

    #[actix_web::test]
    async fn find_by_user_id_with_filters() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let query_started_at_gte: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-01-27T00:00:00Z").unwrap();
        let query_started_at_lte: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-01-27T23:59:59Z").unwrap();
        let action = factory::action(user.id).insert(&db).await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        let _action_track_0 = factory::action_track(user.id)
            .started_at(query_started_at_gte - Duration::seconds(1))
            .duration(Some(120))
            .action_id(action.id)
            .insert(&db)
            .await?;
        let action_track_1 = factory::action_track(user.id)
            .started_at(query_started_at_gte)
            .duration(Some(180))
            .action_id(action.id)
            .insert(&db)
            .await?;
        let action_track_2 = factory::action_track(user.id)
            .started_at(query_started_at_lte)
            .duration(Some(350))
            .action_id(action.id)
            .insert(&db)
            .await?;
        let _action_track_3 = factory::action_track(user.id)
            .started_at(query_started_at_lte + Duration::seconds(1))
            .duration(Some(550))
            .action_id(action.id)
            .insert(&db)
            .await?;
        let _archived_action_track = factory::action_track(user.id)
            .started_at(query_started_at_lte)
            .duration(Some(1))
            .action_id(archived_action.id)
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
                action_id: action.id,
                started_at: action_track_2.started_at,
                ended_at: action_track_2.ended_at,
                duration: action_track_2.duration,
            },
            ActionTrackVisible {
                id: action_track_1.id,
                action_id: action.id,
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
}
