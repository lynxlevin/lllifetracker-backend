use crate::{
    entities::user as user_entity,
    services::memo_mutation::{MemoMutation, UpdateMemo},
    types::{self, CustomDbErr, MemoVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr, TransactionError};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    memo_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    title: Option<String>,
    text: Option<String>,
    date: Option<chrono::NaiveDate>,
    tag_ids: Option<Vec<uuid::Uuid>>,
}

#[tracing::instrument(name = "Updating a memo", skip(db, user, req, path_param))]
#[put("/{memo_id}")]
pub async fn update_memo(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let form = UpdateMemo {
                id: path_param.memo_id,
                title: req.title.clone(),
                text: req.text.clone(),
                date: req.date,
                tag_ids: req.tag_ids.clone(),
                user_id: user.id,
            };
            match MemoMutation::partial_update(&db, form).await {
                Ok(memo) => HttpResponse::Ok().json(MemoVisible {
                    id: memo.id,
                    title: memo.title,
                    text: memo.text,
                    date: memo.date,
                    created_at: memo.created_at,
                    updated_at: memo.updated_at,
                }),
                Err(e) => match e {
                    TransactionError::Transaction(DbErr::Custom(message)) => {
                        match message.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "Memo with this id was not found".to_string(),
                                })
                            }
                        }
                    }
                    e => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                        HttpResponse::InternalServerError().json(types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                },
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test, App, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait, QuerySelect};

    use crate::{
        entities::{memo, memos_tags},
        test_utils,
    };

    use super::*;

    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    enum QueryAs {
        TagId,
    }

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(update_memo).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo =
            test_utils::seed::create_memo(&db, "Memo without tags".to_string(), user.id).await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        let form = RequestBody {
            title: Some("memo after update title".to_string()),
            text: Some("memo after update text".to_string()),
            date: Some(chrono::NaiveDate::from_ymd_opt(2024, 11, 3).unwrap()),
            tag_ids: Some(vec![ambition_tag.id]),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/{}", memo.id))
            .set_json(&form)
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_memo: MemoVisible = test::read_body_json(res).await;
        assert_eq!(returned_memo.title, form.title.clone().unwrap());
        assert_eq!(returned_memo.text, form.text.clone().unwrap());
        assert_eq!(returned_memo.date, form.date.unwrap());
        assert_eq!(returned_memo.created_at, memo.created_at);
        assert!(returned_memo.updated_at > memo.updated_at);

        let updated_memo = memo::Entity::find_by_id(returned_memo.id)
            .filter(memo::Column::Title.eq(form.title.unwrap()))
            .filter(memo::Column::Text.eq(form.text.unwrap()))
            .filter(memo::Column::Date.eq(form.date.unwrap()))
            .filter(memo::Column::UserId.eq(user.id))
            .filter(memo::Column::CreatedAt.eq(returned_memo.created_at))
            .filter(memo::Column::UpdatedAt.eq(returned_memo.updated_at))
            .one(&db)
            .await?;
        assert!(updated_memo.is_some());

        let linked_tag_ids: Vec<uuid::Uuid> = memos_tags::Entity::find()
            .column_as(memos_tags::Column::TagId, QueryAs::TagId)
            .filter(memos_tags::Column::MemoId.eq(returned_memo.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 1);
        assert!(linked_tag_ids.contains(&ambition_tag.id));

        Ok(())
    }

    #[actix_web::test]
    async fn not_found_if_invalid_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", uuid::Uuid::new_v4()))
            .set_json(RequestBody {
                title: None,
                text: None,
                date: None,
                tag_ids: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo =
            test_utils::seed::create_memo(&db, "Memo without tags".to_string(), user.id).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", memo.id))
            .set_json(RequestBody {
                title: None,
                text: None,
                date: None,
                tag_ids: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
