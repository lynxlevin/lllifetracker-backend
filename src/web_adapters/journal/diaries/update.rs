use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::diary_adapter::DiaryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    journal::diaries::{types::DiaryUpdateRequest, update::update_diary},
    UseCaseError,
};
use uuid::Uuid;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    diary_id: Uuid,
}

#[tracing::instrument(name = "Updating a diary", skip(db, user, req, path_param))]
#[put("/{diary_id}")]
pub async fn update_diary_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DiaryUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_diary(
                user.into_inner(),
                req.into_inner(),
                path_param.diary_id,
                DiaryAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
