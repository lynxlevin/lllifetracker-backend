use crate::{
    entities::user as user_entity,
    services::ambition_mutation::AmbitionMutation,
    types::{self, AmbitionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    ambition_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
    description: Option<String>,
}

#[tracing::instrument(name = "Updating an ambition", skip(db, user, req, path_param))]
#[put("/{ambition_id}")]
pub async fn update_ambition(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionMutation::update(
                &db,
                path_param.ambition_id,
                user.id,
                req.name.clone(),
                req.description.clone(),
            )
            .await
            {
                Ok(ambition) => HttpResponse::Ok().json(AmbitionVisible {
                    id: ambition.id,
                    name: ambition.name,
                    description: ambition.description,
                    created_at: ambition.created_at,
                    updated_at: ambition.updated_at,
                }),
                Err(e) => match e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "Ambition with this id was not found".to_string(),
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

    use crate::{entities::ambition, test_utils};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(update_ambition).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let ambition =
            test_utils::seed::create_ambition(&db, "ambition".to_string(), None, user.id).await?;
        let new_name = "ambition_after_update_route".to_string();
        let new_description = Some("edited_description".to_string());

        let req = test::TestRequest::put()
            .uri(&format!("/{}", ambition.id))
            .set_json(RequestBody {
                name: new_name.clone(),
                description: new_description.clone(),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_ambition: AmbitionVisible = test::read_body_json(res).await;
        assert_eq!(returned_ambition.id, ambition.id);
        assert_eq!(returned_ambition.name, new_name.clone());
        assert_eq!(returned_ambition.description, new_description.clone());
        assert_eq!(returned_ambition.created_at, ambition.created_at);
        assert!(returned_ambition.updated_at > ambition.updated_at);

        let updated_ambition = ambition::Entity::find_by_id(ambition.id)
            .filter(ambition::Column::Name.eq(new_name))
            .filter(ambition::Column::Description.eq(new_description))
            .filter(ambition::Column::UserId.eq(user.id))
            .filter(ambition::Column::CreatedAt.eq(returned_ambition.created_at))
            .filter(ambition::Column::UpdatedAt.eq(returned_ambition.updated_at))
            .one(&db)
            .await?;
        assert!(updated_ambition.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_no_description() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let ambition = test_utils::seed::create_ambition(
            &db,
            "ambition".to_string(),
            Some("Original description".to_string()),
            user.id,
        )
        .await?;
        let new_name = "ambition_after_update_route".to_string();

        let req = test::TestRequest::put()
            .uri(&format!("/{}", ambition.id))
            .set_json(RequestBody {
                name: new_name.clone(),
                description: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_ambition: AmbitionVisible = test::read_body_json(res).await;
        assert_eq!(returned_ambition.id, ambition.id);
        assert_eq!(returned_ambition.name, new_name.clone());
        assert!(returned_ambition.description.is_none());
        assert_eq!(returned_ambition.created_at, ambition.created_at);
        assert!(returned_ambition.updated_at > ambition.updated_at);

        let updated_ambition = ambition::Entity::find_by_id(ambition.id)
            .filter(ambition::Column::Name.eq(new_name))
            .filter(ambition::Column::Description.is_null())
            .filter(ambition::Column::UserId.eq(user.id))
            .filter(ambition::Column::CreatedAt.eq(returned_ambition.created_at))
            .filter(ambition::Column::UpdatedAt.eq(returned_ambition.updated_at))
            .one(&db)
            .await?;
        assert!(updated_ambition.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let ambition =
            test_utils::seed::create_ambition(&db, "ambition".to_string(), None, user.id).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", ambition.id))
            .set_json(RequestBody {
                name: "ambition_after_update_route".to_string(),
                description: Some("edited_description".to_string()),
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
