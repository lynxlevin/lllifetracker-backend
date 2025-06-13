use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::desired_state_category_query::DesiredStateCategoryQuery;
use types::DesiredStateCategoryVisible;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's desired_state_categories", skip(db, user))]
#[get("")]
pub async fn list_desired_state_categories(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateCategoryQuery::find_all_by_user_id(&db, user.id).await {
                Ok(categories) => HttpResponse::Ok().json(
                    categories
                        .iter()
                        .map(|category| DesiredStateCategoryVisible::from(category))
                        .collect::<Vec<_>>(),
                ),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
