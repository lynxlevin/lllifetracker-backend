use ::types::DiaryVisible;
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{
    sqlx::error::Error::Database, DbConn, DbErr, RuntimeErr::SqlxError, TransactionError,
};
use services::diary_mutation::{DiaryMutation, NewDiary};
use types::DiaryCreateRequest;

use crate::utils::{response_400, response_401, response_404, response_409, response_500};

#[tracing::instrument(name = "Creating a diary", skip(db, user))]
#[post("")]
pub async fn create_diary(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DiaryCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match _validate_request_body(&req) {
                Ok(_) => {
                    match DiaryMutation::create(
                        &db,
                        NewDiary {
                            text: req.text.clone(),
                            date: req.date,
                            score: req.score,
                            tag_ids: req.tag_ids.clone(),
                            user_id: user.id,
                        },
                    )
                    .await
                    {
                        Ok(diary) => {
                            let res: DiaryVisible = diary.into();
                            HttpResponse::Created().json(res)
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

fn _validate_request_body(req: &DiaryCreateRequest) -> Result<(), String> {
    if req.score.is_some_and(|score| score > 5 || score < 1) {
        return Err("score should be within 1 to 5.".to_string());
    }
    Ok(())
}
