use crate::{
    entities::user as user_entity,
    services::action::Mutation as ActionMutation,
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
            match ActionMutation::update(&db, path_param.action_id, user.id, req.name.clone()).await
            {
                Ok(action) => HttpResponse::Ok().json(ActionVisible {
                    id: action.id,
                    name: action.name,
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

    use crate::{entities::action, test_utils};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(update_action).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn test_happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_user(&db).await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "action_for_update_route".to_string(),
            user.id,
        )
        .await?;
        let new_name = "action_after_update_route".to_string();

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: new_name.clone(),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_action: ActionVisible = test::read_body_json(resp).await;
        assert_eq!(returned_action.id, action.id);
        assert_eq!(returned_action.name, new_name.clone());
        assert_eq!(returned_action.created_at, action.created_at);
        assert!(returned_action.updated_at > action.updated_at);

        let updated_action = action::Entity::find_by_id(action.id)
            .filter(action::Column::Name.eq(new_name))
            .filter(action::Column::UserId.eq(user.id))
            .filter(action::Column::CreatedAt.eq(action.created_at))
            .filter(action::Column::UpdatedAt.eq(returned_action.updated_at))
            .one(&db)
            .await?;
        assert!(updated_action.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn test_unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_user(&db).await?;
        let (action, _) = test_utils::seed::create_action_and_tag(
            &db,
            "action_for_update_route_unauthorized".to_string(),
            user.id,
        )
        .await?;
        let new_name = "action_after_update_route".to_string();

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: new_name.clone(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
