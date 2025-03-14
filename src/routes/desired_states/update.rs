use entities::user as user_entity;
use ::types::{self, CustomDbErr, DesiredStateVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use services::desired_state_mutation::DesiredStateMutation;
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
    description: Option<String>,
}

#[tracing::instrument(name = "Updating an desired_state", skip(db, user, req, path_param))]
#[put("/{desired_state_id}")]
pub async fn update_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateMutation::update(
                &db,
                path_param.desired_state_id,
                user.id,
                req.name.clone(),
                req.description.clone(),
            )
            .await
            {
                Ok(desired_state) => {
                    let res: DesiredStateVisible = desired_state.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => match e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "DesiredState with this id was not found".to_string(),
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

    use entities::desired_state;
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(update_desired_state).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let new_name = "desired_state_after_update".to_string();
        let new_description = "DesiredState after update.".to_string();

        let req = test::TestRequest::put()
            .uri(&format!("/{}", desired_state.id))
            .set_json(RequestBody {
                name: new_name.clone(),
                description: Some(new_description.clone()),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_desired_state: DesiredStateVisible = test::read_body_json(res).await;
        assert_eq!(returned_desired_state.id, desired_state.id);
        assert_eq!(returned_desired_state.name, new_name.clone());
        assert_eq!(
            returned_desired_state.description,
            Some(new_description.clone())
        );
        assert_eq!(returned_desired_state.created_at, desired_state.created_at);
        assert!(returned_desired_state.updated_at > desired_state.updated_at);

        let updated_desired_state = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_desired_state.id, desired_state.id);
        assert_eq!(updated_desired_state.name, new_name.clone());
        assert_eq!(updated_desired_state.description, Some(new_description.clone()));
        assert_eq!(updated_desired_state.archived, desired_state.archived);
        assert_eq!(updated_desired_state.created_at, desired_state.created_at);
        assert_eq!(updated_desired_state.updated_at, returned_desired_state.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", desired_state.id))
            .set_json(RequestBody {
                name: "desired_state".to_string(),
                description: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
