use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::{
    desired_state_category_adapter::{
        DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryOrder,
        DesiredStateCategoryQuery,
    },
    Order::Asc,
};
use entities::user as user_entity;
use sea_orm::DbConn;
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
            match DesiredStateCategoryAdapter::init(&db)
                .filter_eq_user(&user)
                .order_by_ordering_nulls_last(Asc)
                .order_by_id(Asc)
                .get_all()
                .await
            {
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
