use ::types::{self, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::{
    action_query::ActionQuery, desired_state_mutation::DesiredStateMutation,
    desired_state_query::DesiredStateQuery,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    desired_state_id: uuid::Uuid,
    action_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Disconnecting an action from an desired_state",
    skip(db, user, path_param)
)]
#[delete("/{desired_state_id}/actions/{action_id}/connection")]
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
                    match DesiredStateMutation::disconnect_action(
                        &db,
                        path_param.desired_state_id,
                        path_param.action_id,
                    )
                    .await
                    {
                        Ok(_) => HttpResponse::Ok().json(types::SuccessResponse {
                            message: format!(
                                "Successfully disconnected desired_state: {} with action: {}",
                                path_param.desired_state_id, path_param.action_id
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
                    error: "DesiredState or action with the requested ids were not found"
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
    if let Err(e) =
        DesiredStateQuery::find_by_id_and_user_id(db, path_param.desired_state_id, user_id).await
    {
        match e {
            DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                CustomDbErr::NotFound => {}
                _ => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                }
            },
            _ => {
                tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
            }
        }
        return Err(());
    }
    if let Err(e) = ActionQuery::find_by_id_and_user_id(db, path_param.action_id, user_id).await {
        match e {
            DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                CustomDbErr::NotFound => {}
                _ => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                }
            },
            _ => {
                tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
            }
        }
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test, App, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait};

    use entities::desired_states_actions;
    use test_utils::{self, *};

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
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;
        factory::link_desired_state_action(&db, desired_state.id, action.id).await?;

        let req = test::TestRequest::delete()
            .uri(&format!(
                "/{}/actions/{}/connection",
                desired_state.id, action.id
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let connection_in_db = desired_states_actions::Entity::find()
            .filter(desired_states_actions::Column::DesiredStateId.eq(desired_state.id))
            .filter(desired_states_actions::Column::ActionId.eq(action.id))
            .one(&db)
            .await?;
        assert!(connection_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn invalid_desired_state() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(another_user.id).insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!(
                "/{}/actions/{}/connection",
                desired_state.id, action.id
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
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;
        let action = factory::action(another_user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!(
                "/{}/actions/{}/connection",
                desired_state.id, action.id
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
        let desired_state = factory::desired_state(user.id).insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!(
                "/{}/actions/{}/connection",
                desired_state.id, action.id
            ))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
