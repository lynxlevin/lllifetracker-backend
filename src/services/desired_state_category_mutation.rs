use entities::desired_state_category;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, IntoActiveModel, ModelTrait, Set,
};
use uuid::Uuid;

pub struct DesiredStateCategoryMutation;

impl DesiredStateCategoryMutation {
    pub async fn create(
        db: &DbConn,
        user_id: Uuid,
        name: String,
    ) -> Result<desired_state_category::Model, DbErr> {
        desired_state_category::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(user_id),
            name: Set(name),
            ordering: NotSet,
        }
        .insert(db)
        .await
    }

    pub async fn update(
        db: &DbConn,
        category: desired_state_category::Model,
        name: String,
    ) -> Result<desired_state_category::Model, DbErr> {
        let mut category = category.into_active_model();
        category.name = Set(name);
        category.update(db).await
    }

    pub async fn delete(db: &DbConn, category: desired_state_category::Model) -> Result<(), DbErr> {
        category.delete(db).await?;
        Ok(())
    }

    pub async fn bulk_update_ordering(
        db: &DbConn,
        params: Vec<(desired_state_category::Model, Option<i32>)>,
    ) -> Result<(), DbErr> {
        for (category, ordering) in params {
            let mut category = category.into_active_model();
            category.ordering = Set(ordering);
            category.update(db).await?;
        }
        Ok(())
    }
}
