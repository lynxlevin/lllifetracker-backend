use entities::user as user_entity;
use ::types::{self, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use crate::{
    services::{
        ambition_mutation::AmbitionMutation, ambition_query::AmbitionQuery,
        objective_query::ObjectiveQuery,
    },
};
use actix_web::{
    post,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    ambition_id: uuid::Uuid,
    objective_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Connecting an objective to an ambition",
    skip(db, user, path_param)
)]
#[post("/{ambition_id}/objectives/{objective_id}/connection")]
pub async fn connect_objective(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match validate_ownership(&db, user.id, &path_param).await {
                Ok(_) => {
                    match AmbitionMutation::connect_objective(
                        &db,
                        path_param.ambition_id,
                        path_param.objective_id,
                    )
                    .await
                    {
                        Ok(_) => HttpResponse::Ok().json(types::SuccessResponse {
                            message: format!(
                                "Successfully connected ambition: {} with objective: {}",
                                path_param.ambition_id, path_param.objective_id
                            ),
                        }),
                        Err(e) => {
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                            HttpResponse::InternalServerError().json(types::ErrorResponse {
                                error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                            })
                        }
                    }
                }
                Err(_) => HttpResponse::NotFound().json(types::ErrorResponse {
                    error: "Ambition or objective with the requested ids were not found"
                        .to_string(),
                }),
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}

async fn validate_ownership(
    db: &DbConn,
    user_id: uuid::Uuid,
    path_param: &Path<PathParam>,
) -> Result<(), ()> {
    match AmbitionQuery::find_by_id_and_user_id(db, path_param.ambition_id, user_id).await {
        Ok(_) => match ObjectiveQuery::find_by_id_and_user_id(db, path_param.objective_id, user_id)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match e {
                DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                    CustomDbErr::NotFound => Err(()),
                },
                e => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    Err(())
                }
            },
        },
        Err(e) => match e {
            DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                CustomDbErr::NotFound => Err(()),
            },
            e => {
                tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                Err(())
            }
        },
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

    use entities::ambitions_objectives;
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(connect_objective)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;
        let objective = factory::objective(user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/{}/objectives/{}/connection",
                ambition.id, objective.id
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let created_connection = ambitions_objectives::Entity::find()
            .filter(ambitions_objectives::Column::AmbitionId.eq(ambition.id))
            .filter(ambitions_objectives::Column::ObjectiveId.eq(objective.id))
            .one(&db)
            .await?;
        assert!(created_connection.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn invalid_ambition() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(another_user.id).insert(&db).await?;
        let objective = factory::objective(user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/{}/objectives/{}/connection",
                ambition.id, objective.id
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn invalid_objective() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;
        let objective = factory::objective(another_user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/{}/objectives/{}/connection",
                ambition.id, objective.id
            ))
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
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;
        let objective = factory::objective(user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/{}/objectives/{}/connection",
                ambition.id, objective.id
            ))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
