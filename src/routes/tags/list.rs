use crate::{
    entities::user as user_entity,
    services::tag_query::TagQuery,
    types::{self, TagQueryResult, TagType, TagVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing a user's tags.", skip(db, user))]
#[get("")]
pub async fn list_tags(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match TagQuery::find_all_by_user_id(&db, user.id).await {
                Ok(tags) => {
                    let res: Vec<TagVisible> =
                        tags.into_iter().map(|tag| get_tag_visible(tag)).collect();
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

fn get_tag_visible(tag: TagQueryResult) -> TagVisible {
    if let Some(name) = tag.ambition_name.clone() {
        TagVisible {
            id: tag.id,
            name,
            tag_type: TagType::Ambition,
            created_at: tag.created_at,
        }
    } else if let Some(name) = tag.objective_name.clone() {
        TagVisible {
            id: tag.id,
            name,
            tag_type: TagType::Objective,
            created_at: tag.created_at,
        }
    } else if let Some(name) = tag.action_name.clone() {
        TagVisible {
            id: tag.id,
            name,
            tag_type: TagType::Action,
            created_at: tag.created_at,
        }
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

    use crate::test_utils;

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_tags))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (action, action_tag) =
            test_utils::seed::create_action_and_tag(&db, "action".to_string(), None, user.id)
                .await?;
        let (ambition, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        let (objective, objective_tag) =
            test_utils::seed::create_objective_and_tag(&db, "objective".to_string(), None, user.id)
                .await?;
        let _archived_action =
            test_utils::seed::create_action_and_tag(&db, "action".to_string(), None, user.id)
                .await?
                .0
                .archive(&db)
                .await?;
        let _archived_ambition =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?
                .0
                .archive(&db)
                .await?;
        let _archived_objective =
            test_utils::seed::create_objective_and_tag(&db, "objective".to_string(), None, user.id)
                .await?
                .0
                .archive(&db)
                .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<TagVisible> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 3);

        let expected = serde_json::json!([
            {
                "id": ambition_tag.id,
                "name": ambition.name.clone(),
                "tag_type": TagType::Ambition,
                "created_at": ambition_tag.created_at,
            },
            {
                "id": objective_tag.id,
                "name": objective.name.clone(),
                "tag_type": TagType::Objective,
                "created_at": objective_tag.created_at,
            },
            {
                "id": action_tag.id,
                "name": action.name.clone(),
                "tag_type": TagType::Action,
                "created_at": action_tag.created_at,
            },
        ]);

        let body = serde_json::to_value(&body).unwrap();
        assert_eq!(expected, body);

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
