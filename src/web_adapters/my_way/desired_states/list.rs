use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_adapter::DesiredStateAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::desired_states::{list::list_desired_states, types::DesiredStateListQuery};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's desired_states", skip(db, user))]
#[get("")]
pub async fn list_desired_states_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<DesiredStateListQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_desired_states(
                user.into_inner(),
                query.into_inner(),
                DesiredStateAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
