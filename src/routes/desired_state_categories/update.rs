use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::{
    desired_state_category_mutation::DesiredStateCategoryMutation,
    desired_state_category_query::DesiredStateCategoryQuery,
};
use types::{DesiredStateCategoryUpdateRequest, DesiredStateCategoryVisible};
use uuid::Uuid;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    category_id: Uuid,
}

#[tracing::instrument(name = "Updating an desired_state_category", skip(db, user))]
#[put("/{category_id}")]
pub async fn update_desired_state_category(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateCategoryUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let category = match DesiredStateCategoryQuery::find_by_id_and_user_id(
                &db,
                path_param.category_id,
                user.id,
            )
            .await
            {
                Ok(res) => match res {
                    Some(category) => category,
                    None => return response_404("Category not found"),
                },
                Err(e) => return response_500(e),
            };
            match DesiredStateCategoryMutation::update(&db, category, req.name.clone()).await {
                Ok(category) => {
                    let res: DesiredStateCategoryVisible = category.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
