use crate::{
    entities::user as user_entity,
    services::action_mutation::ActionMutation,
    types::{self, ActionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
    description: Option<String>,
}

#[tracing::instrument(name = "Updating an action", skip(db, user, req, path_param))]
#[put("/{action_id}")]
pub async fn update_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::update(
                &db,
                path_param.action_id,
                user.id,
                req.name.clone(),
                req.description.clone(),
            )
            .await
            {
                Ok(action) => HttpResponse::Ok().json(ActionVisible {
                    id: action.id,
                    name: action.name,
                    description: action.description,
                    created_at: action.created_at,
                    updated_at: action.updated_at,
                }),
                Err(e) => match e {
                    DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "Action with this id was not found".to_string(),
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

    use crate::{entities::action, test_utils::{self, factory}};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(update_action).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let new_name = "action_after_update".to_string();
        let new_description = "Action after update.".to_string();

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: new_name.clone(),
                description: Some(new_description.clone()),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_action: ActionVisible = test::read_body_json(res).await;
        assert_eq!(returned_action.id, action.id);
        assert_eq!(returned_action.name, new_name.clone());
        assert_eq!(returned_action.description, Some(new_description.clone()));
        assert_eq!(returned_action.created_at, action.created_at);
        assert!(returned_action.updated_at > action.updated_at);

        let updated_action = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_action.name, new_name.clone());
        assert_eq!(updated_action.description, Some(new_description.clone()));
        assert_eq!(updated_action.user_id, user.id);
        assert_eq!(updated_action.created_at, returned_action.created_at);
        assert_eq!(updated_action.updated_at, returned_action.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: "action_after_update_route".to_string(),
                description: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
