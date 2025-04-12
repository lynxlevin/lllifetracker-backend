use ::types::{self, AmbitionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::ambition_mutation::AmbitionMutation;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    ambition_id: uuid::Uuid,
}

#[tracing::instrument(name = "Archiving an ambition", skip(db, user, path_param))]
#[put("/{ambition_id}/archive")]
pub async fn archive_ambition(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionMutation::archive(&db, path_param.ambition_id, user.id).await {
                Ok(ambition) => {
                    let res: AmbitionVisible = ambition.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => {
                    match &e {
                        DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                return HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "Ambition with this id was not found".to_string(),
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

    use entities::ambition;
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(archive_ambition).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", ambition.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_ambition: AmbitionVisible = test::read_body_json(res).await;
        assert_eq!(returned_ambition.id, ambition.id);
        assert_eq!(returned_ambition.name, ambition.name.clone());
        assert_eq!(returned_ambition.description, ambition.description.clone());
        assert_eq!(returned_ambition.created_at, ambition.created_at);
        assert!(returned_ambition.updated_at > ambition.updated_at);

        let archived_ambition = ambition::Entity::find_by_id(ambition.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(archived_ambition.id, ambition.id);
        assert_eq!(archived_ambition.name, ambition.name.clone());
        assert_eq!(archived_ambition.description, ambition.description.clone());
        assert_eq!(archived_ambition.archived, true);
        assert_eq!(archived_ambition.created_at, ambition.created_at);
        assert_eq!(archived_ambition.updated_at, returned_ambition.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", ambition.id))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
