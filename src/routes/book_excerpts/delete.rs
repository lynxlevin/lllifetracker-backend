use entities::user as user_entity;
use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use services::book_excerpt_mutation::BookExcerptMutation;
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    book_excerpt_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a book excerpt", skip(db, user, path_param))]
#[delete("/{book_excerpt_id}")]
pub async fn delete_book_excerpt(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match BookExcerptMutation::delete(&db, path_param.book_excerpt_id, user.id).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => {
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
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait};

    use entities::{book_excerpt, book_excerpts_tags};
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(delete_book_excerpt)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let book_excerpt = factory::book_excerpt(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_book_excerpt_tag(&db, book_excerpt.id, ambition_tag.id).await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", book_excerpt.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let book_excerpt_in_db = book_excerpt::Entity::find_by_id(book_excerpt.id)
            .one(&db)
            .await?;
        assert!(book_excerpt_in_db.is_none());

        let book_excerpts_tags_in_db = book_excerpts_tags::Entity::find()
            .filter(book_excerpts_tags::Column::BookExcerptId.eq(book_excerpt.id))
            .filter(book_excerpts_tags::Column::TagId.eq(ambition_tag.id))
            .one(&db)
            .await?;
        assert!(book_excerpts_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let book_excerpt = factory::book_excerpt(user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", book_excerpt.id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
