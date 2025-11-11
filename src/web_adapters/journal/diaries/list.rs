use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::diary_adapter::DiaryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::journal::diaries::{list::list_diaries, types::DiaryListQuery};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing user's diaries.", skip(db, user))]
#[get("")]
pub async fn list_diaries_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => match list_diaries(
            user.into_inner(),
            DiaryAdapter::init(&db),
            DiaryListQuery { tag_id_or: None },
        )
        .await
        {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(e) => response_500(e),
        },
        None => response_401(),
    }
}
