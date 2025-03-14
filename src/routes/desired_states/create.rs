use entities::user as user_entity;
use ::types::{self, DesiredStateVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use services::desired_state_mutation::{NewDesiredState, DesiredStateMutation};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
    description: Option<String>,
}

#[tracing::instrument(name = "Creating an desired_state", skip(db, user))]
#[post("")]
pub async fn create_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateMutation::create_with_tag(
                &db,
                NewDesiredState {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(desired_state) => {
                    let res: DesiredStateVisible = desired_state.into();
                    HttpResponse::Created().json(res)
                }
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
    use entities::{desired_state, tag};
    use test_utils::{self, *};

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
                .service(scope("/").service(create_desired_state))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let name = "create_desired_state route happy path".to_string();
        let description = "Create desired_state route happy path.".to_string();

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                name: name.clone(),
                description: Some(description.clone()),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_desired_state: DesiredStateVisible = test::read_body_json(res).await;
        assert_eq!(returned_desired_state.name, name);

        let created_desired_state = desired_state::Entity::find_by_id(returned_desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_desired_state.name, returned_desired_state.name);
        assert_eq!(
            created_desired_state.description,
            returned_desired_state.description
        );
        assert_eq!(created_desired_state.user_id, user.id);
        assert_eq!(created_desired_state.created_at, returned_desired_state.created_at);
        assert_eq!(created_desired_state.updated_at, returned_desired_state.updated_at);

        let created_tag = tag::Entity::find()
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::DesiredStateId.eq(returned_desired_state.id))
            .filter(tag::Column::ActionId.is_null())
            .filter(tag::Column::UserId.eq(user.id))
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

        Ok(())
    }
}
