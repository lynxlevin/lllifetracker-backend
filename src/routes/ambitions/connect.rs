use ::types::{self, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    post,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::{
    ambition_mutation::AmbitionMutation, ambition_query::AmbitionQuery,
    desired_state_query::DesiredStateQuery,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    ambition_id: uuid::Uuid,
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Connecting an desired_state to an ambition",
    skip(db, user, path_param)
)]
#[post("/{ambition_id}/desired_states/{desired_state_id}/connection")]
pub async fn connect_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match validate_ownership(&db, user.id, &path_param).await {
                Ok(_) => {
                    match AmbitionMutation::connect_desired_state(
                        &db,
                        path_param.ambition_id,
                        path_param.desired_state_id,
                    )
                    .await
                    {
                        Ok(_) => HttpResponse::Ok().json(types::SuccessResponse {
                            message: format!(
                                "Successfully connected ambition: {} with desired_state: {}",
                                path_param.ambition_id, path_param.desired_state_id
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
                    error: "Ambition or desired_state with the requested ids were not found"
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
        Ok(_) => match DesiredStateQuery::find_by_id_and_user_id(db, path_param.desired_state_id, user_id)
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

    use entities::ambitions_desired_states;
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(connect_desired_state)
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
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/{}/desired_states/{}/connection",
                ambition.id, desired_state.id
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let created_connection = ambitions_desired_states::Entity::find()
            .filter(ambitions_desired_states::Column::AmbitionId.eq(ambition.id))
            .filter(ambitions_desired_states::Column::DesiredStateId.eq(desired_state.id))
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
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/{}/desired_states/{}/connection",
                ambition.id, desired_state.id
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn invalid_desired_state() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;
        let desired_state = factory::desired_state(another_user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/{}/desired_states/{}/connection",
                ambition.id, desired_state.id
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
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/{}/desired_states/{}/connection",
                ambition.id, desired_state.id
            ))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
