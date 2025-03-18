use entities::user as user_entity;
use ::types::{self, DiaryVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use services::diary_mutation::{DiaryMutation, NewDiary};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

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
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
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

    use entities::{diary, diaries_tags};
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
}
