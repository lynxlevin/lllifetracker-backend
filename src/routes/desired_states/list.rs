use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use serde::Deserialize;
use services::desired_state_query::DesiredStateQuery;

#[derive(Deserialize, Debug)]
struct QueryParam {
    show_archived_only: Option<bool>,
}

#[tracing::instrument(name = "Listing a user's desired_states", skip(db, user))]
#[get("")]
pub async fn list_desired_states(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateQuery::find_all_by_user_id(
                &db,
                user.id,
                query.show_archived_only.unwrap_or(false),
            )
            .await
            {
                Ok(desired_states) => HttpResponse::Ok().json(desired_states),
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
    use ::types::DesiredStateVisible;
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use sea_orm::{entity::prelude::ActiveModelTrait, DbErr};

    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_desired_states))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let desired_state_0 = factory::desired_state(user.id)
            .name("desired_state_0".to_string())
            .insert(&db)
            .await?;
        let desired_state_1 = factory::desired_state(user.id)
            .name("desired_state_1".to_string())
            .insert(&db)
            .await?;
        let _archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_desired_states: Vec<DesiredStateVisible> = test::read_body_json(resp).await;
        assert_eq!(returned_desired_states[0].id, desired_state_0.id);
        assert_eq!(returned_desired_states[0].name, desired_state_0.name);
        assert_eq!(
            returned_desired_states[0].created_at,
            desired_state_0.created_at
        );
        assert_eq!(
            returned_desired_states[0].updated_at,
            desired_state_0.updated_at
        );

        assert_eq!(returned_desired_states[1].id, desired_state_1.id);
        assert_eq!(returned_desired_states[1].name, desired_state_1.name);
        assert_eq!(
            returned_desired_states[1].created_at,
            desired_state_1.created_at
        );
        assert_eq!(
            returned_desired_states[1].updated_at,
            desired_state_1.updated_at
        );

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_show_archived_only() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let _desired_state = factory::desired_state(user.id).insert(&db).await?;
        let archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get()
            .uri("/?show_archived_only=true")
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<DesiredStateVisible> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": archived_desired_state.id,
            "name": archived_desired_state.name,
            "description": archived_desired_state.description,
            "created_at": archived_desired_state.created_at,
            "updated_at": archived_desired_state.updated_at,
        }]);

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
