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
use sea_orm::{DbConn, DbErr};
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
                    let diary =
                        match DiaryAdapter::init(&db)
                            .create(CreateDiaryParams {
                                text: req.text.clone(),
                                date: req.date,
                                score: req.score,
                                user_id: user.id,
                            })
                            .await
                        {
                            Ok(diary) => diary,
                            Err(e) => match &e {
                                DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                                    CustomDbErr::Duplicate => return response_409(
                                        "Another diary record for the same date already exists.",
                                    ),
                                    _ => return response_500(e),
                                },
                                _ => return response_500(e),
                            },
                        };
                    match DiaryAdapter::init(&db)
                        .link_tags(&diary, req.tag_ids.clone())
                        .await
                    {
                        Ok(_) => HttpResponse::Created().json(DiaryVisible::from(diary)),
                        Err(e) => match &e {
                            // FIXME: diary creation should be canceled.
                            DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                                CustomDbErr::NotFound => {
                                    return response_404("One or more of the tag_ids do not exist.")
                                }
                                _ => return response_500(e),
                            },
                            _ => return response_500(e),
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
