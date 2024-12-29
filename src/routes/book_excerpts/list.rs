use crate::{
    entities::user as user_entity,
    services::book_excerpt_query::BookExcerptQuery,
    types::{
        self, BookExcerptVisibleWithTags, BookExcerptWithTagQueryResult, TagType, TagVisible,
        INTERNAL_SERVER_ERROR_MESSAGE,
    },
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing user's book excerpts.", skip(db, user))]
#[get("")]
pub async fn list_book_excerpts(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match BookExcerptQuery::find_all_with_tags_by_user_id(&db, user.id).await {
                Ok(book_excerpts) => {
                    let mut res: Vec<BookExcerptVisibleWithTags> = vec![];
                    for book_excerpt in book_excerpts {
                        if res.is_empty() || res.last().unwrap().id != book_excerpt.id {
                            let mut res_book_excerpt = BookExcerptVisibleWithTags {
                                id: book_excerpt.id,
                                title: book_excerpt.title.clone(),
                                page_number: book_excerpt.page_number,
                                text: book_excerpt.text.clone(),
                                date: book_excerpt.date,
                                created_at: book_excerpt.created_at,
                                updated_at: book_excerpt.updated_at,
                                tags: vec![],
                            };
                            if let Some(tag) = get_tag(&book_excerpt) {
                                res_book_excerpt.push_tag(tag);
                            }
                            res.push(res_book_excerpt);
                        } else {
                            if let Some(tag) = get_tag(&book_excerpt) {
                                let mut last_book_excerpt = res.pop().unwrap();
                                last_book_excerpt.push_tag(tag);
                                res.push(last_book_excerpt);
                            }
                        }
                    }
                    HttpResponse::Ok().json(res)
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

fn get_tag(book_excerpt: &BookExcerptWithTagQueryResult) -> Option<TagVisible> {
    if book_excerpt.tag_id.is_none() {
        return None;
    }

    if let Some(name) = book_excerpt.tag_ambition_name.clone() {
        Some(TagVisible {
            id: book_excerpt.tag_id.unwrap(),
            name,
            tag_type: TagType::Ambition,
            created_at: book_excerpt.tag_created_at.unwrap(),
        })
    } else if let Some(name) = book_excerpt.tag_objective_name.clone() {
        Some(TagVisible {
            id: book_excerpt.tag_id.unwrap(),
            name,
            tag_type: TagType::Objective,
            created_at: book_excerpt.tag_created_at.unwrap(),
        })
    } else if let Some(name) = book_excerpt.tag_action_name.clone() {
        Some(TagVisible {
            id: book_excerpt.tag_id.unwrap(),
            name,
            tag_type: TagType::Action,
            created_at: book_excerpt.tag_created_at.unwrap(),
        })
    } else {
        unimplemented!("Tag without link to Ambition/Objective/Action is not implemented yet.");
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
    use sea_orm::{entity::prelude::*, DbErr};
    use types::TagType;

    use crate::test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_book_excerpts))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let book_excerpt_0 = factory::book_excerpt(user.id)
            .title("book_excerpt_0".to_string())
            .insert(&db)
            .await?;
        let book_excerpt_1 = factory::book_excerpt(user.id)
            .title("book_excerpt_1".to_string())
            .insert(&db)
            .await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (objective, objective_tag) = factory::objective(user.id).insert_with_tag(&db).await?;
        factory::link_book_excerpt_tag(&db, book_excerpt_0.id, ambition_tag.id).await?;
        factory::link_book_excerpt_tag(&db, book_excerpt_1.id, objective_tag.id).await?;
        factory::link_book_excerpt_tag(&db, book_excerpt_1.id, action_tag.id).await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<BookExcerptVisibleWithTags> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);

        let expected_0 = serde_json::json!({
            "id": book_excerpt_1.id,
            "title": book_excerpt_1.title.clone(),
            "page_number": book_excerpt_1.page_number,
            "text": book_excerpt_1.text.clone(),
            "date": book_excerpt_1.date,
            "created_at": book_excerpt_1.created_at,
            "updated_at": book_excerpt_1.updated_at,
            "tags": [
                {
                    "id": objective_tag.id,
                    "name": objective.name,
                    "tag_type": TagType::Objective,
                    "created_at": objective_tag.created_at,
                },
                {
                    "id": action_tag.id,
                    "name": action.name,
                    "tag_type": TagType::Action,
                    "created_at": action_tag.created_at,
                },
            ],
        });

        let body_0 = serde_json::to_value(&body[0]).unwrap();
        assert_eq!(expected_0, body_0);

        let expected_1 = serde_json::json!({
            "id": book_excerpt_0.id,
            "title": book_excerpt_0.title.clone(),
            "page_number": book_excerpt_0.page_number,
            "text": book_excerpt_0.text.clone(),
            "date": book_excerpt_0.date,
            "created_at": book_excerpt_0.created_at,
            "updated_at": book_excerpt_0.updated_at,
            "tags": [
                {
                    "id": ambition_tag.id,
                    "name": ambition.name,
                    "tag_type": TagType::Ambition,
                    "created_at": ambition_tag.created_at,
                },
            ],
        });
        let body_1 = serde_json::to_value(&body[1]).unwrap();
        assert_eq!(expected_1, body_1,);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
