use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::{
    desired_state_category_mutation::DesiredStateCategoryMutation,
    desired_state_category_query::DesiredStateCategoryQuery,
};
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
            // NOTE: Readability takes precedence over performance because this endpoint won't be called often.
            let category = match DesiredStateCategoryQuery::find_by_id_and_user_id(
                &db,
                path_param.category_id,
                user.id,
            )
            .await
            {
                Ok(res) => match res {
                    Some(category) => category,
                    None => return HttpResponse::NoContent().finish(),
                },
                Err(e) => return response_500(e),
            };
            match DesiredStateCategoryMutation::delete(&db, category).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
