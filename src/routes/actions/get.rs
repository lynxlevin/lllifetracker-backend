use ::types::{self, ActionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::action_query::ActionQuery;

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an action", skip(data, user))]
#[get("/{action_id}")]
pub async fn get_action(
    data: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionQuery::find_by_id_and_user_id(&data, path_param.action_id, user.id).await {
                Ok(action) => {
                    let res: ActionVisible = action.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => {
                    match &e {
                        DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
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
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test, App, HttpMessage,
    };
    use sea_orm::{entity::prelude::ActiveModelTrait, DbErr};

    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(get_action).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::get()
            .uri(&format!("/{}", action.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let res: ActionVisible = test::read_body_json(resp).await;
        assert_eq!(res, ActionVisible::from(action));

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::get()
            .uri(&format!("/{}", action.id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
