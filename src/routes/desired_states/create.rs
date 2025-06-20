use ::types::DesiredStateVisible;
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_adapter::{
    CreateDesiredStateParams, DesiredStateAdapter, DesiredStateMutation,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::desired_state_category_query::DesiredStateCategoryQuery;
use types::DesiredStateCreateRequest;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating an desired_state", skip(db, user))]
#[post("")]
pub async fn create_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let category_id = match req.category_id {
                Some(category_id) => match DesiredStateCategoryQuery::find_by_id_and_user_id(
                    &db,
                    category_id,
                    user.id,
                )
                .await
                {
                    Ok(res) => res.and(Some(category_id)),
                    Err(e) => return response_500(e),
                },
                None => None,
            };
            match DesiredStateAdapter::init(&db)
                .create_with_tag(CreateDesiredStateParams {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    category_id,
                    user_id: user.id,
                })
                .await
            {
                Ok(desired_state) => {
                    HttpResponse::Created().json(DesiredStateVisible::from(desired_state))
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
