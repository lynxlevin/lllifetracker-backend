use ::types::DiaryVisible;
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::{
    diary_adapter::{CreateDiaryParams, DiaryAdapter, DiaryMutation},
    CustomDbErr,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr, TransactionError};
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
                    // MYMEMO: Extract more logics from db_adapter
                    match DiaryAdapter::init(&db)
                        .create(CreateDiaryParams {
                            text: req.text.clone(),
                            date: req.date,
                            score: req.score,
                            tag_ids: req.tag_ids.clone(),
                            user_id: user.id,
                        })
                        .await
                    {
                        Ok(diary) => {
                            let res: DiaryVisible = diary.into();
                            HttpResponse::Created().json(res)
                        }
                        Err(e) => match &e {
                            TransactionError::Transaction(e) => match e {
                                DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                                    CustomDbErr::Duplicate => response_409(
                                        "Another diary record for the same date already exists.",
                                    ),
                                    CustomDbErr::NotFound => {
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
