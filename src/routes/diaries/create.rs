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

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    text: Option<String>,
    date: chrono::NaiveDate,
    score: Option<i16>,
    tag_ids: Vec<uuid::Uuid>,
}

#[tracing::instrument(name = "Creating a diary", skip(db, user))]
#[post("")]
pub async fn create_diary(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
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

fn _validate_request_body(req: &RequestBody) -> Result<(), String> {
    if req.score.is_some_and(|score| score > 5 || score < 1) {
        return Err("score should be within 1 to 5.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait, QuerySelect};

    use entities::{diaries_tags, diary};
    use test_utils::{self, *};

    use super::*;

    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    enum QueryAs {
        TagId,
    }

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(create_diary))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let (_, tag_0) = factory::action(user.id)
            .name("action_0".to_string())
            .insert_with_tag(&db)
            .await?;
        let (_, tag_1) = factory::action(user.id)
            .name("action_1".to_string())
            .insert_with_tag(&db)
            .await?;

        let diary_text = Some("This is a new diary for testing create method.".to_string());
        let today = chrono::Utc::now().date_naive();
        let diary_score = Some(2);
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                text: diary_text.clone(),
                date: today,
                score: diary_score,
                tag_ids: vec![tag_0.id, tag_1.id],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_diary: DiaryVisible = test::read_body_json(res).await;
        assert_eq!(returned_diary.text, diary_text.clone());
        assert_eq!(returned_diary.date, today);
        assert_eq!(returned_diary.score, diary_score);

        let created_diary = diary::Entity::find_by_id(returned_diary.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_diary.text, diary_text.clone());
        assert_eq!(created_diary.date, today);
        assert_eq!(created_diary.score, diary_score);
        assert_eq!(created_diary.user_id, user.id);

        let linked_tag_ids: Vec<uuid::Uuid> = diaries_tags::Entity::find()
            .column_as(diaries_tags::Column::TagId, QueryAs::TagId)
            .filter(diaries_tags::Column::DiaryId.eq(returned_diary.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 2);
        assert!(linked_tag_ids.contains(&tag_0.id));
        assert!(linked_tag_ids.contains(&tag_1.id));

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                text: None,
                date: chrono::Utc::now().date_naive(),
                score: None,
                tag_ids: vec![],
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }

    #[actix_web::test]
    async fn conflict_if_duplicate_exists() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let _existing_diary = factory::diary(user.id)
            .date(chrono::NaiveDate::from_ymd_opt(2025, 3, 19).unwrap())
            .insert(&db)
            .await?;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                text: None,
                date: chrono::NaiveDate::from_ymd_opt(2025, 3, 19).unwrap(),
                score: None,
                tag_ids: vec![],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CONFLICT);

        Ok(())
    }

    #[actix_web::test]
    async fn not_found_on_non_existent_tag_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let today = chrono::Utc::now().date_naive();

        let non_existent_tag_req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                text: None,
                date: today,
                score: None,
                tag_ids: vec![uuid::Uuid::new_v4()],
            })
            .to_request();
        non_existent_tag_req.extensions_mut().insert(user.clone());
        let res = test::call_service(&app, non_existent_tag_req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn validation_errors() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let today = chrono::Utc::now().date_naive();

        let score_too_large_req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                text: None,
                date: today,
                score: Some(6),
                tag_ids: vec![],
            })
            .to_request();
        score_too_large_req.extensions_mut().insert(user.clone());
        let res = test::call_service(&app, score_too_large_req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        let score_too_small_req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                text: None,
                date: today,
                score: Some(0),
                tag_ids: vec![],
            })
            .to_request();
        score_too_small_req.extensions_mut().insert(user.clone());
        let res = test::call_service(&app, score_too_small_req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }
}
