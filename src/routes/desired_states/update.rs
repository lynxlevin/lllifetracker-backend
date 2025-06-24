use ::types::DesiredStateVisible;
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::{
    desired_state_adapter::{
        DesiredStateAdapter, DesiredStateFilter, DesiredStateMutation, DesiredStateQuery,
        UpdateDesiredStateParams,
    },
    desired_state_category_adapter::{
        DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryQuery,
    },
};
use entities::user as user_entity;
use sea_orm::DbConn;
use types::DesiredStateUpdateRequest;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating an desired_state", skip(db, user, req, path_param))]
#[put("/{desired_state_id}")]
pub async fn update_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let category_id = match req.category_id {
                Some(category_id) => match DesiredStateCategoryAdapter::init(&db)
                    .filter_eq_user(&user)
                    .get_by_id(category_id)
                    .await
                {
                    Ok(res) => res.and(Some(category_id)),
                    Err(e) => return response_500(e),
                },
                None => None,
            };
            let desired_state = match DesiredStateAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.desired_state_id)
                .await
            {
                Ok(desired_state) => match desired_state {
                    Some(desired_state) => desired_state,
                    None => return response_404("DesiredState with this id was not found"),
                },
                Err(e) => return response_500(e),
            };
            match DesiredStateAdapter::init(&db)
                .update(
                    desired_state,
                    UpdateDesiredStateParams {
                        name: req.name.clone(),
                        description: req.description.clone(),
                        category_id,
                    },
                )
                .await
            {
                Ok(desired_state) => {
                    HttpResponse::Ok().json(DesiredStateVisible::from(desired_state))
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
