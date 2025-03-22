use ::types::{self, ActionVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::action_mutation::ActionMutation;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
    description: Option<String>,
    trackable: Option<bool>,
    color: Option<String>,
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
            match _validate_request_body(&req) {
                Ok(_) => {
                    match ActionMutation::update(
                        &db,
                        path_param.action_id,
                        user.id,
                        req.name.clone(),
                        req.description.clone(),
                        req.trackable,
                        req.color.clone(),
                    )
                    .await
                    {
                        Ok(action) => {
                            let res: ActionVisible = action.into();
                            HttpResponse::Ok().json(res)
                        }
                        Err(e) => {
                            match &e {
                                DbErr::Custom(message) => {
                                    match message.parse::<CustomDbErr>().unwrap() {
                                        CustomDbErr::NotFound => {
                                            return HttpResponse::NotFound().json(
                                                types::ErrorResponse {
                                                    error: "Action with this id was not found"
                                                        .to_string(),
                                                },
                                            )
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                            HttpResponse::InternalServerError().json(types::ErrorResponse {
                                error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                            })
                        }
                    }
                }
                Err(e) => HttpResponse::BadRequest().json(types::ErrorResponse { error: e }),
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}

fn _validate_request_body(req: &RequestBody) -> Result<(), String> {
    if let Some(color) = &req.color {
        if color.len() != 7 {return Err("color must be 7 characters long.".to_string())}
        if !color.starts_with('#') {return Err("color must be hex color code.".to_string())}
        for c in color.split_at(1).1.chars() {
            if !c.is_ascii_hexdigit() {return Err("color must be hex color code.".to_string())}
        }
    }

    Ok(())
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
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(update_action).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let new_name = "action_after_update".to_string();
        let new_description = "Action after update.".to_string();
        let new_trackable = false;
        let new_color = "#ffffff".to_string();

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: new_name.clone(),
                description: Some(new_description.clone()),
                trackable: Some(new_trackable),
                color: Some(new_color.clone()),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_action: ActionVisible = test::read_body_json(res).await;
        assert_eq!(returned_action.id, action.id);
        assert_eq!(returned_action.name, new_name.clone());
        assert_eq!(returned_action.description, Some(new_description.clone()));
        assert_eq!(returned_action.trackable, new_trackable);
        assert_eq!(returned_action.color, new_color.clone());
        assert_eq!(returned_action.created_at, action.created_at);
        assert!(returned_action.updated_at > action.updated_at);

        let updated_action = action::Entity::find_by_id(action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_action.name, new_name.clone());
        assert_eq!(updated_action.description, Some(new_description.clone()));
        assert_eq!(updated_action.trackable, new_trackable);
        assert_eq!(updated_action.color, new_color.clone());
        assert_eq!(updated_action.user_id, user.id);
        assert_eq!(updated_action.created_at, returned_action.created_at);
        assert_eq!(updated_action.updated_at, returned_action.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: "action_after_update_route".to_string(),
                description: None,
                trackable: None,
                color: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }

    #[actix_web::test]
    async fn validation_errors() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let long_name = "#1234567".to_string();
        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: "action_after_update_route".to_string(),
                description: None,
                trackable: None,
                color: Some(long_name),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        let short_name = "#12345".to_string();
        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: "action_after_update_route".to_string(),
                description: None,
                trackable: None,
                color: Some(short_name),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        let bad_format = "$ffffff".to_string();
        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: "action_after_update_route".to_string(),
                description: None,
                trackable: None,
                color: Some(bad_format),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        let bad_character = "#gggggg".to_string();
        let req = test::TestRequest::put()
            .uri(&format!("/{}", action.id))
            .set_json(RequestBody {
                name: "action_after_update_route".to_string(),
                description: None,
                trackable: None,
                color: Some(bad_character),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }
}
