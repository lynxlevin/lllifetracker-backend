use entities::user as user_entity;
use ::types::{self, ActionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use crate::{
    services::action_mutation::ActionMutation,
};
use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Archiving an action", skip(db, user, path_param))]
#[put("/{action_id}/archive")]
pub async fn archive_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::archive(&db, path_param.action_id, user.id).await {
                Ok(action) => {
                    let res: ActionVisible = action.into();
                    HttpResponse::Ok().json(res)
                }
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

    use entities::action;
    use crate::{
        test_utils::{self, *},
    };

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(archive_action).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", action.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_action: ActionVisible = test::read_body_json(res).await;
        assert_eq!(returned_action.id, action.id);
        assert_eq!(returned_action.name, action.name.clone());
        assert_eq!(returned_action.description, action.description.clone());
        assert_eq!(returned_action.created_at, action.created_at);
        assert!(returned_action.updated_at > action.updated_at);

        let archived_action = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(archived_action.name, action.name.clone());
        assert_eq!(archived_action.description, action.description.clone());
        assert_eq!(archived_action.archived, true);
        assert_eq!(archived_action.user_id, user.id);
        assert_eq!(archived_action.created_at, returned_action.created_at);
        assert_eq!(archived_action.updated_at, returned_action.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", action.id))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
