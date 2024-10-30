use crate::{
    entities::user as user_entity,
    services::ambition_mutation::{AmbitionMutation, NewAmbition},
    types::{self, AmbitionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
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
                Ok(ambition) => HttpResponse::Created().json(AmbitionVisible {
                    id: ambition.id,
                    name: ambition.name,
                    description: ambition.description,
                    created_at: ambition.created_at,
                    updated_at: ambition.updated_at,
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
    use crate::{
        entities::{ambition, tag},
        test_utils,
    };

    use super::*;
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, Error, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, ColumnTrait, DbErr, EntityTrait};

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
        let user = test_utils::seed::create_active_user(&db).await?;
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

        let returned_ambition: AmbitionVisible = test::read_body_json(res).await;
        assert_eq!(returned_ambition.name, name.clone());
        assert_eq!(returned_ambition.description, description.clone());

        let created_ambition = ambition::Entity::find_by_id(returned_ambition.id)
            .filter(ambition::Column::Name.eq(name))
            .filter(ambition::Column::Description.eq(description))
            .filter(ambition::Column::UserId.eq(user.id))
            .filter(ambition::Column::CreatedAt.eq(returned_ambition.created_at))
            .filter(ambition::Column::UpdatedAt.eq(returned_ambition.updated_at))
            .one(&db)
            .await?;
        assert!(created_ambition.is_some());

        let created_tag = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::AmbitionId.eq(returned_ambition.id))
            .filter(tag::Column::ObjectiveId.is_null())
            .filter(tag::Column::ActionId.is_null())
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_no_description() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
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

        let returned_ambition: AmbitionVisible = test::read_body_json(res).await;
        assert_eq!(returned_ambition.name, name.clone());
        assert!(returned_ambition.description.is_none());

        let created_ambition = ambition::Entity::find_by_id(returned_ambition.id)
            .filter(ambition::Column::Name.eq(name))
            .filter(ambition::Column::Description.is_null())
            .filter(ambition::Column::UserId.eq(user.id))
            .filter(ambition::Column::CreatedAt.eq(returned_ambition.created_at))
            .filter(ambition::Column::UpdatedAt.eq(returned_ambition.updated_at))
            .one(&db)
            .await?;
        assert!(created_ambition.is_some());

        let created_tag = tag::Entity::find()
            .filter(tag::Column::UserId.eq(user.id))
            .filter(tag::Column::AmbitionId.eq(returned_ambition.id))
            .filter(tag::Column::ObjectiveId.is_null())
            .filter(tag::Column::ActionId.is_null())
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

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
