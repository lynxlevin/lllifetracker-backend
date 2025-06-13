use entities::desired_state_category;
use sea_orm::{
    sea_query::NullOrdering::Last, ColumnTrait, DbConn, DbErr, EntityTrait, Order::Asc,
    QueryFilter, QueryOrder,
};
use uuid::Uuid;

pub struct DesiredStateCategoryQuery;

impl DesiredStateCategoryQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: Uuid,
    ) -> Result<Vec<desired_state_category::Model>, DbErr> {
        desired_state_category::Entity::find()
            .filter(desired_state_category::Column::UserId.eq(user_id))
            .order_by_with_nulls(desired_state_category::Column::Ordering, Asc, Last)
            .order_by_asc(desired_state_category::Column::Id)
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        category_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<desired_state_category::Model>, DbErr> {
        desired_state_category::Entity::find_by_id(category_id)
            .filter(desired_state_category::Column::UserId.eq(user_id))
            .one(db)
            .await
    }
    pub async fn find_in_ids_by_user_id(
        db: &DbConn,
        category_ids: Vec<Uuid>,
        user_id: Uuid,
    ) -> Result<Vec<desired_state_category::Model>, DbErr> {
        desired_state_category::Entity::find()
            .filter(desired_state_category::Column::Id.is_in(category_ids))
            .filter(desired_state_category::Column::UserId.eq(user_id))
            .all(db)
            .await
    }
}
