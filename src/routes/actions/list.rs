use crate::{
    entities::user as user_entity,
    services::action::Query as ActionQuery,
    types::{self, ActionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing a user's actions", skip(db, user))]
#[get("")]
pub async fn list_actions(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionQuery::find_all_by_user_id(&db, user.id).await {
                Ok(actions) => HttpResponse::Ok().json(
                    actions
                        .iter()
                        .map(|action| ActionVisible {
                            id: action.id,
                            name: action.name.clone(),
                            created_at: action.created_at,
                            updated_at: action.updated_at,
                        })
                        .collect::<Vec<ActionVisible>>(),
                ),
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
    use sea_orm::{entity::prelude::*, DbErr};

    use crate::test_utils;

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_actions))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn test_happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::get_or_create_user(&db).await?;
        let (action_1, _) = test_utils::seed::get_or_create_action_and_tag(
            &db,
            "action_for_get_1".to_string(),
            user.id,
        )
        .await;
        let (action_2, _) = test_utils::seed::get_or_create_action_and_tag(
            &db,
            "action_for_get_2".to_string(),
            user.id,
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_actions: Vec<ActionVisible> = test::read_body_json(resp).await;
        assert_eq!(returned_actions[0].id, action_1.id);
        assert_eq!(returned_actions[0].name, action_1.name);
        assert_eq!(returned_actions[0].created_at, action_1.created_at);
        assert_eq!(returned_actions[0].updated_at, action_1.updated_at);

        assert_eq!(returned_actions[1].id, action_2.id);
        assert_eq!(returned_actions[1].name, action_2.name);
        assert_eq!(returned_actions[1].created_at, action_2.created_at);
        assert_eq!(returned_actions[1].updated_at, action_2.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn test_unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
