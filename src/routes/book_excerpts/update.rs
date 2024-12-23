use crate::{
    entities::user as user_entity,
    services::book_excerpt_mutation::{BookExcerptMutation, UpdateBookExcerpt},
    types::{self, BookExcerptVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr, TransactionError};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    book_excerpt_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    title: Option<String>,
    page_number: Option<i16>,
    text: Option<String>,
    date: Option<chrono::NaiveDate>,
    tag_ids: Option<Vec<uuid::Uuid>>,
}

#[tracing::instrument(name = "Updating a book excerpt", skip(db, user, req, path_param))]
#[put("/{book_excerpt_id}")]
pub async fn update_book_excerpt(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let form = UpdateBookExcerpt {
                id: path_param.book_excerpt_id,
                title: req.title.clone(),
                page_number: req.page_number,
                text: req.text.clone(),
                date: req.date,
                tag_ids: req.tag_ids.clone(),
                user_id: user.id,
            };
            match BookExcerptMutation::partial_update(&db, form).await {
                Ok(book_excerpt) => HttpResponse::Ok().json(BookExcerptVisible {
                    id: book_excerpt.id,
                    title: book_excerpt.title,
                    page_number: book_excerpt.page_number,
                    text: book_excerpt.text,
                    date: book_excerpt.date,
                    created_at: book_excerpt.created_at,
                    updated_at: book_excerpt.updated_at,
                }),
                Err(e) => match e {
                    TransactionError::Transaction(DbErr::Custom(message)) => {
                        match message.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "book excerpt with this id was not found".to_string(),
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
        entities::{book_excerpt, book_excerpts_tags},
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
        test::init_service(
            App::new()
                .service(update_book_excerpt)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "book excerpt without tags".to_string(),
            user.id,
        )
        .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        let form = RequestBody {
            title: Some("book excerpt after update title".to_string()),
            page_number: Some(998),
            text: Some("book excerpt after update text".to_string()),
            date: Some(chrono::NaiveDate::from_ymd_opt(2024, 11, 3).unwrap()),
            tag_ids: Some(vec![ambition_tag.id]),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/{}", book_excerpt.id))
            .set_json(&form)
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_book_excerpt: BookExcerptVisible = test::read_body_json(res).await;
        assert_eq!(returned_book_excerpt.title, form.title.clone().unwrap());
        assert_eq!(returned_book_excerpt.page_number, form.page_number.unwrap());
        assert_eq!(returned_book_excerpt.text, form.text.clone().unwrap());
        assert_eq!(returned_book_excerpt.date, form.date.unwrap());
        assert_eq!(returned_book_excerpt.created_at, book_excerpt.created_at);
        assert!(returned_book_excerpt.updated_at > book_excerpt.updated_at);

        let updated_book_excerpt = book_excerpt::Entity::find_by_id(returned_book_excerpt.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_book_excerpt.title, form.title.unwrap());
        assert_eq!(updated_book_excerpt.page_number, form.page_number.unwrap());
        assert_eq!(updated_book_excerpt.text, form.text.unwrap());
        assert_eq!(updated_book_excerpt.date, form.date.unwrap());
        assert_eq!(updated_book_excerpt.user_id, user.id);
        assert_eq!(
            updated_book_excerpt.created_at,
            returned_book_excerpt.created_at
        );
        assert_eq!(
            updated_book_excerpt.updated_at,
            returned_book_excerpt.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = book_excerpts_tags::Entity::find()
            .column_as(book_excerpts_tags::Column::TagId, QueryAs::TagId)
            .filter(book_excerpts_tags::Column::BookExcerptId.eq(returned_book_excerpt.id))
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
                page_number: None,
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
        let book_excerpt = test_utils::seed::create_book_excerpt(
            &db,
            "book excerpt without tags".to_string(),
            user.id,
        )
        .await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", book_excerpt.id))
            .set_json(RequestBody {
                title: None,
                page_number: None,
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
