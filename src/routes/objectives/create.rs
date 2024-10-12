use crate::{
    entities::user as user_entity,
    services::objective::{Mutation as ObjectiveMutation, NewObjective},
    types::{self, ObjectiveVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
}

#[tracing::instrument(name = "Creating an objective", skip(db, user))]
#[post("")]
pub async fn create_objective(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ObjectiveMutation::create_with_tag(
                &db,
                NewObjective {
                    name: req.name.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(objective) => HttpResponse::Created().json(ObjectiveVisible {
                    id: objective.id,
                    name: objective.name,
                    created_at: objective.created_at,
                    updated_at: objective.updated_at,
                }),
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
    use crate::{
        entities::{objective, tag},
        test_utils,
    };

    use super::*;
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, Error, HttpMessage,
    };
    use sea_orm::prelude::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(create_objective))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let name = "create_objective route happy path".to_string();

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody { name: name.clone() })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_objective: ObjectiveVisible = test::read_body_json(res).await;
        assert_eq!(returned_objective.name, name);

        let created_objective = objective::Entity::find_by_id(returned_objective.id)
            .filter(objective::Column::Name.eq(returned_objective.name))
            .filter(objective::Column::UserId.eq(user.id))
            .filter(objective::Column::CreatedAt.eq(returned_objective.created_at))
            .filter(objective::Column::UpdatedAt.eq(returned_objective.updated_at))
            .one(&db)
            .await?;
        assert!(created_objective.is_some());

        let created_tag = tag::Entity::find()
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::ObjectiveId.eq(returned_objective.id))
            .filter(tag::Column::ActionId.is_null())
            .filter(tag::Column::UserId.eq(user.id))
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

        Ok(())
    }
}
