use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    get,
    web::{self, Data, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use serde::Deserialize;
use services::ambition_query::AmbitionQuery;

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
            match AmbitionQuery::find_all_by_user_id(
                &db,
                user.id,
                query.show_archived_only.unwrap_or(false),
            )
            .await
            {
                Ok(ambitions) => HttpResponse::Ok().json(ambitions),
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}
