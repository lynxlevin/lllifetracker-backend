use ::types::{self, ActionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::action_mutation::ActionMutation;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Restoring an action from archive", skip(db, user, path_param))]
#[put("/{action_id}/unarchive")]
pub async fn unarchive_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::unarchive(&db, path_param.action_id, user.id).await {
                Ok(action) => {
                    let res: ActionVisible = action.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => {
                    match &e {
                        DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                return HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "Action with this id was not found".to_string(),
                                })
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
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
    use sea_orm::{entity::prelude::ActiveModelTrait, DbErr, EntityTrait};

    use entities::action;
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(unarchive_action).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).archived(true).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/unarchive", action.id))
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

        let restored_action = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(restored_action.name, action.name.clone());
        assert_eq!(restored_action.description, action.description.clone());
        assert_eq!(restored_action.archived, false);
        assert_eq!(restored_action.user_id, user.id);
        assert_eq!(restored_action.created_at, returned_action.created_at);
        assert_eq!(restored_action.updated_at, returned_action.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).archived(true).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/unarchive", action.id))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
