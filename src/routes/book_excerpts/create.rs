use crate::{
    entities::user as user_entity,
    services::book_excerpt_mutation::{BookExcerptMutation, NewBookExcerpt},
    types::{self, BookExcerptVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    title: String,
    page_number: i16,
    text: String,
    date: chrono::NaiveDate,
    tag_ids: Vec<uuid::Uuid>,
}

#[tracing::instrument(name = "Creating a book excerpt", skip(db, user))]
#[post("")]
pub async fn create_book_excerpt(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match BookExcerptMutation::create(
                &db,
                NewBookExcerpt {
                    title: req.title.clone(),
                    page_number: req.page_number,
                    text: req.text.clone(),
                    date: req.date,
                    tag_ids: req.tag_ids.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(book_excerpt) => HttpResponse::Created().json(BookExcerptVisible {
                    id: book_excerpt.id,
                    title: book_excerpt.title,
                    page_number: book_excerpt.page_number,
                    text: book_excerpt.text,
                    date: book_excerpt.date,
                    created_at: book_excerpt.created_at,
                    updated_at: book_excerpt.updated_at,
                }),
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

    use crate::{
        entities::{book_excerpt, book_excerpts_tags},
        test_utils::{self, *},
    };

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
                .service(scope("/").service(create_book_excerpt))
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

        let book_excerpt_title = "New book excerpt".to_string();
        let page_number = 12;
        let book_excerpt_text = "This is a new book excerpt for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                title: book_excerpt_title.clone(),
                page_number: page_number,
                text: book_excerpt_text.clone(),
                date: today,
                tag_ids: vec![tag_0.id, tag_1.id],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_book_excerpt: BookExcerptVisible = test::read_body_json(res).await;
        assert_eq!(returned_book_excerpt.title, book_excerpt_title.clone());
        assert_eq!(returned_book_excerpt.page_number, page_number);
        assert_eq!(returned_book_excerpt.text, book_excerpt_text.clone());
        assert_eq!(returned_book_excerpt.date, today);

        let created_book_excerpt = book_excerpt::Entity::find_by_id(returned_book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_book_excerpt.title, book_excerpt_title.clone());
        assert_eq!(created_book_excerpt.page_number, page_number);
        assert_eq!(created_book_excerpt.text, book_excerpt_text.clone());
        assert_eq!(created_book_excerpt.date, today);
        assert_eq!(created_book_excerpt.user_id, user.id);
        assert_eq!(
            created_book_excerpt.created_at,
            returned_book_excerpt.created_at
        );
        assert_eq!(
            created_book_excerpt.updated_at,
            returned_book_excerpt.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = book_excerpts_tags::Entity::find()
            .column_as(book_excerpts_tags::Column::TagId, QueryAs::TagId)
            .filter(book_excerpts_tags::Column::BookExcerptId.eq(returned_book_excerpt.id))
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
                title: "New BookExcerpt".to_string(),
                page_number: 1,
                text: "This is a new book excerpt for testing create method.".to_string(),
                date: chrono::Utc::now().date_naive(),
                tag_ids: vec![],
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
