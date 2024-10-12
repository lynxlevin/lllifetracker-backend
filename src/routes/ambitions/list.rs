use crate::{
    entities::user as user_entity,
    services::ambition::Query as AmbitionQuery,
    types::{self, AmbitionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing a user's ambitions", skip(db, user))]
#[get("")]
pub async fn list_ambitions(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionQuery::find_all_by_user_id(&db, user.id).await {
                Ok(ambitions) => HttpResponse::Ok().json(
                    ambitions
                        .iter()
                        .map(|ambition| AmbitionVisible {
                            id: ambition.id,
                            name: ambition.name.clone(),
                            description: ambition.description.clone(),
                            created_at: ambition.created_at,
                            updated_at: ambition.updated_at,
                        })
                        .collect::<Vec<AmbitionVisible>>(),
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
                .service(scope("/").service(list_ambitions))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (ambition_1, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "ambition_for_get_1".to_string(),
            None,
            user.id,
        )
        .await?;
        let (ambition_2, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "ambition_for_get_2".to_string(),
            Some("ambition_for_get_2".to_string()),
            user.id,
        )
        .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_ambitions: Vec<AmbitionVisible> = test::read_body_json(resp).await;
        assert_eq!(returned_ambitions[0].id, ambition_1.id);
        assert_eq!(returned_ambitions[0].name, ambition_1.name);
        assert_eq!(returned_ambitions[0].description, ambition_1.description);
        assert_eq!(returned_ambitions[0].created_at, ambition_1.created_at);
        assert_eq!(returned_ambitions[0].updated_at, ambition_1.updated_at);

        assert_eq!(returned_ambitions[1].id, ambition_2.id);
        assert_eq!(returned_ambitions[1].name, ambition_2.name);
        assert_eq!(returned_ambitions[1].description, ambition_2.description);
        assert_eq!(returned_ambitions[1].created_at, ambition_2.created_at);
        assert_eq!(returned_ambitions[1].updated_at, ambition_2.updated_at);

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
