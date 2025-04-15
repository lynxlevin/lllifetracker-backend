use types::DiaryCreateRequest;
use ::types::{self, DiaryVisible, INTERNAL_SERVER_ERROR_MESSAGE};
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
                        Err(e) => {
                            match &e {
                                TransactionError::Transaction(e) => match e {
                                    DbErr::Query(SqlxError(Database(e))) => match e.constraint() {
                                        Some("diaries_user_id_date_unique_index") => {
                                            return HttpResponse::Conflict().json(types::ErrorResponse {
                                                error:
                                                    "Another diary record for the same date already exists."
                                                        .to_string(),
                                            })
                                        }
                                        _ => {}
                                    },
                                    DbErr::Exec(SqlxError(Database(e))) => match e.constraint() {
                                        Some("fk-diaries_tags-tag_id") => {
                                            return HttpResponse::NotFound().json(types::ErrorResponse {
                                                error: "One or more of the tag_ids do not exist."
                                                    .to_string(),
                                            })
                                        }
                                        _ => {}
                                    },
                                    _ => {}
                                },
                                _ => {}
                            }
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                            HttpResponse::InternalServerError().json(types::ErrorResponse {
                                error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                            })
                        }
                    }
                }
                Err(e) => HttpResponse::BadRequest().json(types::ErrorResponse { error: e }),
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}

fn _validate_request_body(req: &DiaryCreateRequest) -> Result<(), String> {
    if req.score.is_some_and(|score| score > 5 || score < 1) {
        return Err("score should be within 1 to 5.".to_string());
    }
    Ok(())
}
