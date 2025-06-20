use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use db_adapters::{
    desired_state_adapter::{
        DesiredStateAdapter, DesiredStateFilter, DesiredStateOrder, DesiredStateQuery,
    },
    Order::Asc,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use serde::Deserialize;
use types::DesiredStateVisible;

use crate::utils::{response_401, response_500};

#[derive(Deserialize, Debug)]
struct QueryParam {
    show_archived_only: Option<bool>,
}

#[tracing::instrument(name = "Listing a user's desired_states", skip(db, user))]
#[get("")]
pub async fn list_desired_states(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateAdapter::init(&db)
                .filter_eq_user(&user)
                .filter_eq_archived(query.show_archived_only.unwrap_or(false))
                .order_by_ordering_nulls_last(Asc)
                .order_by_created_at(Asc)
                .get_all()
                .await
            {
                Ok(desired_states) => HttpResponse::Ok().json(
                    desired_states
                        .iter()
                        .map(|desired_state| DesiredStateVisible::from(desired_state))
                        .collect::<Vec<_>>(),
                ),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
