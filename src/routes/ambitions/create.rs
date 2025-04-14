use ::types::{self, AmbitionVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::ambition_mutation::{AmbitionMutation, NewAmbition};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
    description: Option<String>,
}

#[tracing::instrument(name = "Creating an ambition", skip(db, user))]
#[post("")]
pub async fn create_ambition(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionMutation::create_with_tag(
                &db,
                NewAmbition {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(ambition) => {
                    let res: AmbitionVisible = ambition.into();
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
    use entities::{ambition, tag};
    use test_utils::{self, *};

    use super::*;
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, Error, HttpMessage,
    };
    use sea_orm::{
        entity::prelude::ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter,
    };

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(create_ambition))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let app = init_app(db.clone()).await;

        let name = "Test create_ambition route".to_string();
        let description = Some("Test description".to_string());
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                name: name.clone(),
                description: description.clone(),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let res: AmbitionVisible = test::read_body_json(res).await;
        assert_eq!(res.name, name.clone());
        assert_eq!(res.description, description.clone());

        let ambition_in_db = ambition::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db.user_id, user.id);
        assert_eq!(AmbitionVisible::from(ambition_in_db), res);

        let tag_in_db = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::AmbitionId.eq(res.id))
            .filter(tag::Column::DesiredStateId.is_null())
            .filter(tag::Column::ActionId.is_null())
            .one(&db)
            .await?;
        assert!(tag_in_db.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_no_description() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let app = init_app(db.clone()).await;

        let name = "Test create_ambition route".to_string();
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                name: name.clone(),
                description: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let res: AmbitionVisible = test::read_body_json(res).await;
        assert_eq!(res.name, name.clone());
        assert!(res.description.is_none());

        let ambition_in_db = ambition::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(ambition_in_db.user_id, user.id);
        assert_eq!(AmbitionVisible::from(ambition_in_db), res);

        let tag_in_db = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::AmbitionId.eq(res.id))
            .filter(tag::Column::DesiredStateId.is_null())
            .filter(tag::Column::ActionId.is_null())
            .one(&db)
            .await?;
        assert!(tag_in_db.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                name: "Test create_ambition not logged in".to_string(),
                description: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
