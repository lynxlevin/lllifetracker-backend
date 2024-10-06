use crate::{
    entities::user as user_entity,
    services::action::{Mutation as ActionMutation, NewAction},
    types::{self, ActionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
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

#[tracing::instrument(name = "Creating an action", skip(db, user))]
#[post("")]
pub async fn create_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::create_with_tag(
                &db,
                NewAction {
                    name: req.name.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(action) => HttpResponse::Ok().json(ActionVisible {
                    id: action.id,
                    name: action.name,
                    created_at: action.created_at,
                    updated_at: action.updated_at,
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
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait};
    use user_entity::Model;

    use crate::{
        entities::{action, tag, user},
        startup::get_database_connection,
    };

    use super::*;

    #[actix_web::test]
    async fn main() -> Result<(), DbErr> {
        dotenvy::from_filename(".env.test").unwrap();
        let db = get_database_connection().await;
        Migrator::up(&db, None).await.unwrap();

        let user = user::Entity::find()
            .filter(user::Column::Email.eq("test@test.com".to_string()))
            .one(&db)
            .await?
            .unwrap();

        let app = test::init_service(
            App::new()
                .service(scope("/").service(create_action))
                .app_data(Data::new(db.clone())),
        )
        .await;

        test_happy_path(&app, &db, user).await?;
        test_unauthorized_if_not_logged_in(&app).await?;

        Ok(())
    }

    async fn test_happy_path(
        app: &impl Service<Request, Response = ServiceResponse, Error = actix_web::Error>,
        db: &DbConn,
        user: Model,
    ) -> Result<(), DbErr> {
        let action_name = "Test create_action route".to_string();
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                name: action_name.clone(),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_action: ActionVisible = test::read_body_json(resp).await;
        assert_eq!(returned_action.name, action_name.clone());

        let created_action = action::Entity::find_by_id(returned_action.id)
            .filter(action::Column::Name.eq(action_name))
            .filter(action::Column::UserId.eq(user.id))
            .filter(action::Column::CreatedAt.eq(returned_action.created_at))
            .filter(action::Column::UpdatedAt.eq(returned_action.updated_at))
            .one(db)
            .await?;
        assert_ne!(created_action, None);

        let created_tag = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::ActionId.eq(returned_action.id))
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::ObjectiveId.is_null())
            .one(db)
            .await?;
        assert_ne!(created_tag, None);

        Ok(())
    }

    async fn test_unauthorized_if_not_logged_in(
        app: &impl Service<Request, Response = ServiceResponse, Error = actix_web::Error>,
    ) -> Result<(), DbErr> {
        let action_name = "Test create_action not logged in".to_string();
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                name: action_name.clone(),
            })
            .to_request();

        let resp = test::call_service(app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
