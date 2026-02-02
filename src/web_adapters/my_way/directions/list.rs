use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use db_adapters::direction_adapter::DirectionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::directions::{list::list_directions, types::DirectionListQuery};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's directions", skip(db, user))]
#[get("")]
pub async fn list_directions_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<DirectionListQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_directions(
                user.into_inner(),
                query.into_inner(),
                DirectionAdapter::init(&db),
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
