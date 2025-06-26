use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::diary_adapter::DiaryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    journal::diaries::{create::create_diary, types::DiaryCreateRequest},
    UseCaseError,
};

use crate::utils::{response_400, response_401, response_404, response_409, response_500};

#[tracing::instrument(name = "Creating a diary", skip(db, user))]
#[post("")]
pub async fn create_diary_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DiaryCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_diary(user.into_inner(), req.into_inner(), DiaryAdapter::init(&db)).await {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => match &e {
                    UseCaseError::BadRequest(message) => response_400(message),
                    UseCaseError::NotFound(message) => response_404(message),
                    UseCaseError::Conflict(message) => response_409(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
