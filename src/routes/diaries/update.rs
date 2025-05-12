use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{
    sqlx::error::Error::Database, DbConn, DbErr, RuntimeErr::SqlxError, TransactionError,
};
use services::diary_mutation::{DiaryMutation, UpdateDiary};
use types::{CustomDbErr, DiaryUpdateRequest, DiaryVisible};

use crate::utils::{response_400, response_401, response_404, response_409, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    diary_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating a diary", skip(db, user, req, path_param))]
#[put("/{diary_id}")]
pub async fn update_diary(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DiaryUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match _validate_request_body(&req) {
                Ok(_) => {
                    let form = UpdateDiary {
                        id: path_param.diary_id,
                        text: req.text.clone(),
                        date: req.date,
                        score: req.score,
                        tag_ids: req.tag_ids.clone(),
                        user_id: user.id,
                        update_keys: req.update_keys.clone(),
                    };
                    match DiaryMutation::partial_update(&db, form).await {
                        Ok(diary) => {
                            let res: DiaryVisible = diary.into();
                            HttpResponse::Ok().json(res)
                        }
                        Err(e) => match &e {
                            TransactionError::Transaction(e) => match e {
                                DbErr::Query(SqlxError(Database(e))) => match e.constraint() {
                                    Some("diaries_user_id_date_unique_index") => response_409(
                                        "Another diary record for the same date already exists.",
                                    ),
                                    _ => response_500(e),
                                },
                                DbErr::Exec(SqlxError(Database(e))) => match e.constraint() {
                                    Some("fk-diaries_tags-tag_id") => {
                                        response_404("One or more of the tag_ids do not exist.")
                                    }
                                    _ => response_500(e),
                                },
                                DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                                    CustomDbErr::NotFound => {
                                        response_404("Diary with this id was not found")
                                    }
                                    _ => response_500(e),
                                },
                                _ => response_500(e),
                            },
                            _ => response_500(e),
                        },
                    }
                }
                Err(e) => response_400(&e),
            }
        }
        None => response_401(),
    }
}

fn _validate_request_body(req: &DiaryUpdateRequest) -> Result<(), String> {
    if req.score.is_some_and(|score| score > 5 || score < 1) {
        return Err("score should be within 1 to 5.".to_string());
    }
    Ok(())
}
