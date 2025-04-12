use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    get,
    web::{self, Data, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use serde::Deserialize;
use services::ambition_query::AmbitionQuery;

#[derive(Deserialize, Debug)]
struct QueryParam {
    show_archived_only: Option<bool>,
}

#[tracing::instrument(name = "Listing a user's ambitions", skip(db, user))]
#[get("")]
pub async fn list_ambitions(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: web::Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionQuery::find_all_by_user_id(
                &db,
                user.id,
                query.show_archived_only.unwrap_or(false),
            )
            .await
            {
                Ok(ambitions) => HttpResponse::Ok().json(ambitions),
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
    use ::types::AmbitionVisible;
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
                .service(scope("/").service(list_ambitions))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id)
            .name("ambition_0".to_string())
            .insert(&db)
            .await?;
        let ambition_1 = factory::ambition(user.id)
            .name("ambition1".to_string())
            .description(Some("ambition1".to_string()))
            .insert(&db)
            .await?;
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_ambitions: Vec<AmbitionVisible> = test::read_body_json(resp).await;
        assert_eq!(returned_ambitions.len(), 2);
        assert_eq!(
            serde_json::to_value(&returned_ambitions[0]).unwrap(),
            serde_json::json!({
                "id": ambition_0.id,
                "name": ambition_0.name,
                "description": ambition_0.description,
                "created_at": ambition_0.created_at,
                "updated_at": ambition_0.updated_at,
            })
        );
        assert_eq!(
            serde_json::to_value(&returned_ambitions[1]).unwrap(),
            serde_json::json!({
                "id": ambition_1.id,
                "name": ambition_1.name,
                "description": ambition_1.description,
                "created_at": ambition_1.created_at,
                "updated_at": ambition_1.updated_at,
            })
        );

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_show_archived_only() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let _ambition = factory::ambition(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get()
            .uri("/?show_archived_only=true")
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<AmbitionVisible> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": archived_ambition.id,
            "name": archived_ambition.name,
            "description": archived_ambition.description,
            "created_at": archived_ambition.created_at,
            "updated_at": archived_ambition.updated_at,
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
