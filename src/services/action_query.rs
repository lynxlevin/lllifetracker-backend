use ::types::{ActionVisible, CustomDbErr};
use entities::action;
use sea_orm::{
    sea_query::NullOrdering::Last, ColumnTrait, DbConn, DbErr, EntityTrait, Order::Asc,
    QueryFilter, QueryOrder,
};

pub struct ActionQuery;

impl ActionQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
        show_archived_only: bool,
    ) -> Result<Vec<ActionVisible>, DbErr> {
        action::Entity::find()
            .filter(action::Column::UserId.eq(user_id))
            .filter(action::Column::Archived.eq(show_archived_only))
            .order_by_with_nulls(action::Column::Ordering, Asc, Last)
            .order_by_asc(action::Column::CreatedAt)
            .into_partial_model::<ActionVisible>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<action::Model, DbErr> {
        action::Entity::find_by_id(action_id)
            .filter(action::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use test_utils::{self, *};

    use sea_orm::ActiveModelTrait;

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action_0 = factory::action(user.id)
            .name("action_0".to_string())
            .insert(&db)
            .await?;
        let action_1 = factory::action(user.id)
            .name("action_1".to_string())
            .description(Some("Action_1".to_string()))
            .ordering(Some(2))
            .insert(&db)
            .await?;
        let action_2 = factory::action(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        let _archived_action = factory::action(user.id).archived(true).insert(&db).await?;

        let res = ActionQuery::find_all_by_user_id(&db, user.id, false).await?;

        let expected = [
            ActionVisible::from(action_2),
            ActionVisible::from(action_1),
            ActionVisible::from(action_0),
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_by_user_id_show_archived_only() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let _action = factory::action(user.id).insert(&db).await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;

        let res = ActionQuery::find_all_by_user_id(&db, user.id, true).await?;

        let expected = [ActionVisible {
            id: archived_action.id,
            name: archived_action.name,
            description: archived_action.description,
            trackable: archived_action.trackable,
            color: archived_action.color,
            track_type: archived_action.track_type,
            created_at: archived_action.created_at,
            updated_at: archived_action.updated_at,
        }];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }
}
