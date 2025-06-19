use actix_web::{
    get,
    web::{self, Data, ReqData},
    HttpResponse,
};
use db_adapters::ambition_adapter::{AmbitionAdapter, AmbitionFilter, AmbitionQuery};
use entities::user as user_entity;
use sea_orm::DbConn;
use serde::Deserialize;
use types::AmbitionVisible;

use crate::utils::{response_401, response_500};

#[derive(Deserialize, Debug)]
struct QueryParam {
    show_archived_only: Option<bool>,
}

#[tracing::instrument(name = "Listing a user's ambitions", skip(db, user))]
#[get("")]
pub async fn list_ambitions(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: web::Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionAdapter::init(&db)
                .filter_eq_user(&user)
                .filter_eq_archived(query.show_archived_only.unwrap_or(false))
                .get_all()
                .await
            {
                Ok(ambitions) => HttpResponse::Ok().json(
                    ambitions
                        .iter()
                        .map(|ambition| AmbitionVisible::from(ambition))
                        .collect::<Vec<_>>(),
                ),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
