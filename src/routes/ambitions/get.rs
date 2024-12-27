use crate::{
    entities::user as user_entity,
    services::ambition_query::AmbitionQuery,
    types::{self, AmbitionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    ambition_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an ambition", skip(db, user))]
#[get("/{ambition_id}")]
pub async fn get_ambition(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionQuery::find_by_id_and_user_id(&db, path_param.ambition_id, user.id).await
            {
                Ok(ambition) => HttpResponse::Ok().json(AmbitionVisible {
                    id: ambition.id,
                    name: ambition.name,
                    description: ambition.description,
                    created_at: ambition.created_at,
                    updated_at: ambition.updated_at,
                }),
                Err(e) => match e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "Ambition with this id was not found".to_string(),
                            })
                        }
                    },
                    e => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                        HttpResponse::InternalServerError().json(types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                },
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
        http, test, App, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, DbErr};

    use crate::test_utils;

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(get_ambition).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let ambition = test_utils::seed::create_ambition(
            &db,
            "ambition".to_string(),
            Some("ambition".to_string()),
            user.id,
        )
        .await?;

        let req = test::TestRequest::get()
            .uri(&format!("/{}", ambition.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_ambition: AmbitionVisible = test::read_body_json(resp).await;
        assert_eq!(returned_ambition.id, ambition.id);
        assert_eq!(returned_ambition.name, ambition.name);
        assert_eq!(returned_ambition.description, ambition.description);
        assert_eq!(returned_ambition.created_at, ambition.created_at);
        assert_eq!(returned_ambition.updated_at, ambition.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let ambition =
            test_utils::seed::create_ambition(&db, "ambition".to_string(), None, user.id).await?;

        let req = test::TestRequest::get()
            .uri(&format!("/{}", ambition.id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
