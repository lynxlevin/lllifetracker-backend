use crate::{
    entities::user as user_entity,
    services::{
        action_query::ActionQuery, objective_mutation::ObjectiveMutation,
        objective_query::ObjectiveQuery,
    },
    types::{self, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    objective_id: uuid::Uuid,
    action_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Disconnecting an action from an objective",
    skip(db, user, path_param)
)]
#[delete("/{objective_id}/actions/{action_id}/connection")]
pub async fn disconnect_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match validate_ownership(&db, user.id, &path_param).await {
                Ok(_) => {
                    match ObjectiveMutation::disconnect_action(
                        &db,
                        path_param.objective_id,
                        path_param.action_id,
                    )
                    .await
                    {
                        Ok(_) => HttpResponse::Ok().json(types::SuccessResponse {
                            message: format!(
                                "Successfully disconnected objective: {} with action: {}",
                                path_param.objective_id, path_param.action_id
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
                    error: "Objective or action with the requested ids were not found".to_string(),
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
    match ObjectiveQuery::find_by_id_and_user_id(db, path_param.objective_id, user_id).await {
        Ok(_) => match ActionQuery::find_by_id_and_user_id(db, path_param.action_id, user_id).await
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
    use sea_orm::{entity::prelude::*, ActiveValue::Set, DbErr, EntityTrait};

    use crate::{entities::objectives_actions, test_utils::{self, factory}};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(disconnect_action)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let objective =
            test_utils::seed::create_objective(&db, "objective".to_string(), None, user.id).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let _connection = objectives_actions::ActiveModel {
            objective_id: Set(objective.id),
            action_id: Set(action.id),
        }
        .insert(&db)
        .await?;

        let req = test::TestRequest::delete()
            .uri(&format!(
                "/{}/actions/{}/connection",
                objective.id, action.id
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let connection_in_db = objectives_actions::Entity::find()
            .filter(objectives_actions::Column::ObjectiveId.eq(objective.id))
            .filter(objectives_actions::Column::ActionId.eq(action.id))
            .one(&db)
            .await?;
        assert!(connection_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn invalid_objective() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let another_user = test_utils::seed::create_active_user(&db).await?;
        let objective =
            test_utils::seed::create_objective(&db, "objective".to_string(), None, another_user.id)
                .await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!(
                "/{}/actions/{}/connection",
                objective.id, action.id
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn invalid_action() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let another_user = test_utils::seed::create_active_user(&db).await?;
        let objective =
            test_utils::seed::create_objective(&db, "objective".to_string(), None, user.id).await?;
        let action = factory::action(another_user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!(
                "/{}/actions/{}/connection",
                objective.id, action.id
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
        let user = test_utils::seed::create_active_user(&db).await?;
        let objective =
            test_utils::seed::create_objective(&db, "objective".to_string(), None, user.id).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!(
                "/{}/actions/{}/connection",
                objective.id, action.id
            ))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
