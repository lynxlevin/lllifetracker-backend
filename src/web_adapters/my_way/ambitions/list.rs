use actix_web::{
    get,
    web::{self, Data, ReqData},
    HttpResponse,
};
use db_adapters::ambition_adapter::AmbitionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::ambitions::{list::list_ambitions, types::AmbitionListQuery};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's ambitions", skip(db, user))]
#[get("")]
pub async fn list_ambitions_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: web::Query<AmbitionListQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_ambitions(
                user.into_inner(),
                query.into_inner(),
                AmbitionAdapter::init(&db),
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
