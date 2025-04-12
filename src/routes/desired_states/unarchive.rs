use ::types::{self, CustomDbErr, DesiredStateVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::desired_state_mutation::DesiredStateMutation;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Restoring an desired_state from unarchive",
    skip(db, user, path_param)
)]
#[put("/{desired_state_id}/unarchive")]
pub async fn unarchive_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateMutation::unarchive(&db, path_param.desired_state_id, user.id).await {
                Ok(desired_state) => {
                    let res: DesiredStateVisible = desired_state.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => {
                    match &e {
                        DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                return HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "DesiredState with this id was not found".to_string(),
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

    use entities::desired_state;
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(unarchive_desired_state)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/unarchive", desired_state.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_desired_state: DesiredStateVisible = test::read_body_json(res).await;
        assert_eq!(returned_desired_state.id, desired_state.id);
        assert_eq!(returned_desired_state.name, desired_state.name.clone());
        assert_eq!(
            returned_desired_state.description,
            desired_state.description.clone()
        );
        assert_eq!(returned_desired_state.created_at, desired_state.created_at);
        assert!(returned_desired_state.updated_at > desired_state.updated_at);

        let restored_desired_state = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(restored_desired_state.id, desired_state.id);
        assert_eq!(restored_desired_state.name, desired_state.name.clone());
        assert_eq!(
            restored_desired_state.description,
            desired_state.description.clone()
        );
        assert_eq!(restored_desired_state.archived, false);
        assert_eq!(restored_desired_state.created_at, desired_state.created_at);
        assert_eq!(
            restored_desired_state.updated_at,
            returned_desired_state.updated_at
        );

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/unarchive", desired_state.id))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
