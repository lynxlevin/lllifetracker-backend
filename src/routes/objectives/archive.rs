use crate::{
    entities::user as user_entity,
    services::objective_mutation::ObjectiveMutation,
    types::{self, CustomDbErr, ObjectiveVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    objective_id: uuid::Uuid,
}

#[tracing::instrument(name = "Archiving an objective", skip(db, user, path_param))]
#[put("/{objective_id}/archive")]
pub async fn archive_objective(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ObjectiveMutation::archive(&db, path_param.objective_id, user.id).await {
                Ok(objective) => HttpResponse::Ok().json(ObjectiveVisible {
                    id: objective.id,
                    name: objective.name,
                    description: objective.description,
                    created_at: objective.created_at,
                    updated_at: objective.updated_at,
                }),
                Err(e) => match e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "Objective with this id was not found".to_string(),
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
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
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

    use crate::{entities::objective, test_utils};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(archive_objective)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, _) =
            test_utils::seed::create_objective_and_tag(&db, "objective".to_string(), None, user.id)
                .await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", objective.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_objective: ObjectiveVisible = test::read_body_json(res).await;
        assert_eq!(returned_objective.id, objective.id);
        assert_eq!(returned_objective.name, objective.name.clone());
        assert_eq!(
            returned_objective.description,
            objective.description.clone()
        );
        assert_eq!(returned_objective.created_at, objective.created_at);
        assert!(returned_objective.updated_at > objective.updated_at);

        let archived_objective = objective::Entity::find_by_id(objective.id).one(&db).await?.unwrap();
        assert_eq!(archived_objective.id, objective.id);
        assert_eq!(archived_objective.name, objective.name.clone());
        assert_eq!(archived_objective.description, objective.description.clone());
        assert_eq!(archived_objective.archived, true);
        assert_eq!(archived_objective.created_at, objective.created_at);
        assert_eq!(archived_objective.updated_at, returned_objective.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective, _) =
            test_utils::seed::create_objective_and_tag(&db, "objective".to_string(), None, user.id)
                .await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", objective.id))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
