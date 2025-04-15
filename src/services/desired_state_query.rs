use ::types::{CustomDbErr, DesiredStateVisible};
use entities::desired_state;
use sea_orm::{
    sea_query::NullOrdering::Last, ColumnTrait, DbConn, DbErr, EntityTrait, Order::Asc,
    QueryFilter, QueryOrder,
};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewDesiredState {
    pub name: String,
    pub user_id: uuid::Uuid,
}

pub struct DesiredStateQuery;

impl DesiredStateQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
        show_archived_only: bool,
    ) -> Result<Vec<DesiredStateVisible>, DbErr> {
        desired_state::Entity::find()
            .filter(desired_state::Column::UserId.eq(user_id))
            .filter(desired_state::Column::Archived.eq(show_archived_only))
            .order_by_with_nulls(desired_state::Column::Ordering, Asc, Last)
            .order_by_asc(desired_state::Column::CreatedAt)
            .into_partial_model::<DesiredStateVisible>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<desired_state::Model, DbErr> {
        desired_state::Entity::find_by_id(desired_state_id)
            .filter(desired_state::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::ActiveModelTrait;
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state_0 = factory::desired_state(user.id)
            .name("desired_state_0".to_string())
            .insert(&db)
            .await?;
        let desired_state_1 = factory::desired_state(user.id)
            .name("desired_state_1".to_string())
            .description(Some("desired_state_1".to_string()))
            .ordering(Some(2))
            .insert(&db)
            .await?;
        let desired_state_2 = factory::desired_state(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        let _archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = DesiredStateQuery::find_all_by_user_id(&db, user.id, false).await?;

        let expected = [
            DesiredStateVisible {
                id: desired_state_2.id,
                name: desired_state_2.name,
                description: desired_state_2.description,
                created_at: desired_state_2.created_at,
                updated_at: desired_state_2.updated_at,
            },
            DesiredStateVisible {
                id: desired_state_1.id,
                name: desired_state_1.name,
                description: desired_state_1.description,
                created_at: desired_state_1.created_at,
                updated_at: desired_state_1.updated_at,
            },
            DesiredStateVisible {
                id: desired_state_0.id,
                name: desired_state_0.name,
                description: desired_state_0.description,
                created_at: desired_state_0.created_at,
                updated_at: desired_state_0.updated_at,
            },
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
        let _desired_state = factory::desired_state(user.id).insert(&db).await?;
        let archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = DesiredStateQuery::find_all_by_user_id(&db, user.id, true).await?;

        let expected = [DesiredStateVisible {
            id: archived_desired_state.id,
            name: archived_desired_state.name,
            description: archived_desired_state.description,
            created_at: archived_desired_state.created_at,
            updated_at: archived_desired_state.updated_at,
        }];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }
}
