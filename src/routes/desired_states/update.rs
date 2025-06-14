use ::types::{CustomDbErr, DesiredStateVisible};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::{
    desired_state_category_query::DesiredStateCategoryQuery,
    desired_state_mutation::DesiredStateMutation,
};
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
            match DesiredStateMutation::update(
                &db,
                path_param.desired_state_id,
                user.id,
                req.name.clone(),
                req.description.clone(),
                category_id,
            )
            .await
            {
                Ok(desired_state) => {
                    let res: DesiredStateVisible = desired_state.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => match &e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            response_404("DesiredState with this id was not found")
                        }
                        _ => response_500(e),
                    },
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
