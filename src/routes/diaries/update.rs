use ::types::{self, CustomDbErr, DiaryVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{
    sqlx::error::{Error::Database, ErrorKind},
    DbConn, DbErr,
    RuntimeErr::SqlxError,
    TransactionError,
};
use services::diary_mutation::{DiaryKey, DiaryMutation, UpdateDiary};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    diary_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub score: Option<i16>,
    pub tag_ids: Vec<uuid::Uuid>,
    pub update_keys: Vec<DiaryKey>,
}

#[tracing::instrument(name = "Updating a diary", skip(db, user, req, path_param))]
#[put("/{diary_id}")]
pub async fn update_diary(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
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
                Err(e) => {
                    match &e {
                        TransactionError::Transaction(e) => match e {
                            DbErr::Query(SqlxError(Database(e))) => match e.kind() {
                                ErrorKind::UniqueViolation => {
                                    return HttpResponse::Conflict().json(types::ErrorResponse {
                                        error:
                                            "Another diary record for the same date already exists."
                                                .to_string(),
                                    })
                                }
                                _ => {}
                            },
                            DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                                CustomDbErr::NotFound => {
                                    return HttpResponse::NotFound().json(types::ErrorResponse {
                                        error: "Diary with this id was not found".to_string(),
                                    })
                                }
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
        test::init_service(App::new().service(update_diary).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let (_, tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let form = RequestBody {
            text: None,
            date: chrono::NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
            score: None,
            tag_ids: vec![tag.id],
            update_keys: vec![
                DiaryKey::Text,
                DiaryKey::Date,
                DiaryKey::Score,
                DiaryKey::TagIds,
            ],
        };

        let req = test::TestRequest::put()
            .uri(&format!("/{}", diary.id))
            .set_json(&form)
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_diary: DiaryVisible = test::read_body_json(res).await;
        assert_eq!(returned_diary.text, form.text.clone());
        assert_eq!(returned_diary.date, form.date);
        assert_eq!(returned_diary.score, form.score);

        let updated_diary = diary::Entity::find_by_id(returned_diary.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_diary.text, form.text.clone());
        assert_eq!(updated_diary.date, form.date);
        assert_eq!(updated_diary.score, form.score);
        assert_eq!(updated_diary.user_id, user.id);

        let linked_tag_ids: Vec<uuid::Uuid> = diaries_tags::Entity::find()
            .column_as(diaries_tags::Column::TagId, QueryAs::TagId)
            .filter(diaries_tags::Column::DiaryId.eq(returned_diary.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 1);
        assert!(linked_tag_ids.contains(&tag.id));

        Ok(())
    }

    #[actix_web::test]
    async fn not_found_if_invalid_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", uuid::Uuid::new_v4()))
            .set_json(RequestBody {
                text: None,
                date: chrono::Utc::now().date_naive(),
                score: None,
                tag_ids: vec![],
                update_keys: vec![],
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
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", diary.id))
            .set_json(RequestBody {
                text: None,
                date: chrono::Utc::now().date_naive(),
                score: None,
                tag_ids: vec![],
                update_keys: vec![],
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
        let diary = factory::diary(user.id).insert(&db).await?;
        let _existing_diary = factory::diary(user.id)
            .date(chrono::NaiveDate::from_ymd_opt(2025, 3, 19).unwrap())
            .insert(&db)
            .await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", diary.id))
            .set_json(RequestBody {
                text: None,
                date: chrono::NaiveDate::from_ymd_opt(2025, 3, 19).unwrap(),
                score: None,
                tag_ids: vec![],
                update_keys: vec![DiaryKey::Date],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CONFLICT);

        Ok(())
    }
}
