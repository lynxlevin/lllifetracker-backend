use crate::{
    entities::user as user_entity,
    services::action_mutation::{ActionMutation, NewAction},
    types::{self, ActionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
    description: Option<String>,
}

#[tracing::instrument(name = "Creating an action", skip(db, user))]
#[post("")]
pub async fn create_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::create_with_tag(
                &db,
                NewAction {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(action) => HttpResponse::Created().json(ActionVisible {
                    id: action.id,
                    name: action.name,
                    description: action.description,
                    created_at: action.created_at,
                    updated_at: action.updated_at,
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
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait};

    use crate::{
        entities::{action, tag},
        test_utils::{self, *},
    };

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(create_action))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let app = init_app(db.clone()).await;

        let name = "create_action".to_string();
        let description = "Create action.".to_string();
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                name: name.clone(),
                description: Some(description.clone()),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_action: ActionVisible = test::read_body_json(res).await;
        assert_eq!(returned_action.name, name.clone());
        assert_eq!(returned_action.description, Some(description.clone()));

        let created_action = action::Entity::find_by_id(returned_action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_action.name, name);
        assert_eq!(created_action.description, Some(description));
        assert_eq!(created_action.archived, false);
        assert_eq!(created_action.user_id, user.id);
        assert_eq!(created_action.created_at, returned_action.created_at);
        assert_eq!(created_action.updated_at, returned_action.updated_at);

        let created_tag = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::ActionId.eq(returned_action.id))
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::ObjectiveId.is_null())
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                name: "Test create_action not logged in".to_string(),
                description: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
