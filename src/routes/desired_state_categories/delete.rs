use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_category_adapter::{
    DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryMutation,
    DesiredStateCategoryQuery,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use uuid::Uuid;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    category_id: Uuid,
}

#[tracing::instrument(name = "Deleting an desired_state_category", skip(db, user))]
#[delete("/{category_id}")]
pub async fn delete_desired_state_category(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let category = match DesiredStateCategoryAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.category_id)
                .await
            {
                Ok(category) => match category {
                    Some(category) => category,
                    None => return HttpResponse::NoContent().into(),
                },
                Err(e) => return response_500(e),
            };
            match DesiredStateCategoryAdapter::init(&db)
                .delete(category)
                .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
