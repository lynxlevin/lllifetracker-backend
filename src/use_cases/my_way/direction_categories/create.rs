use crate::{
    my_way::direction_categories::types::{
        DirectionCategoryCreateRequest, DirectionCategoryVisible,
    },
    UseCaseError,
};
use db_adapters::direction_category_adapter::{
    CreateDirectionCategoryParams, DirectionCategoryAdapter, DirectionCategoryFilter,
    DirectionCategoryMutation, DirectionCategoryQuery,
};
use entities::user as user_entity;

pub async fn create_direction_category<'a>(
    user: user_entity::Model,
    params: DirectionCategoryCreateRequest,
    category_adapter: DirectionCategoryAdapter<'a>,
) -> Result<DirectionCategoryVisible, UseCaseError> {
    let category_count = category_adapter
        .clone()
        .filter_eq_user(&user)
        .get_count()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    category_adapter
        .create(CreateDirectionCategoryParams {
            name: params.name.clone(),
            ordering: Some(
                (category_count + 1)
                    .try_into()
                    .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?,
            ),
            user_id: user.id,
        })
        .await
        .map(|category| DirectionCategoryVisible::from(category))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
