use std::vec;

use crate::{
    entities::user as user_entity,
    services::ambition_query::AmbitionQuery,
    types::{
        self, ActionVisible, AmbitionVisibleWithLinks,
        AmbitionWithLinksQueryResult, ObjectiveVisibleWithActions, INTERNAL_SERVER_ERROR_MESSAGE,
    },
};
use actix_web::{
    get,
    web::{self, Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct QueryParam {
    links: Option<bool>,
}

#[tracing::instrument(name = "Listing a user's ambitions", skip(db, user))]
#[get("")]
pub async fn list_ambitions(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: web::Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            if query.links.unwrap_or(false) {
                match AmbitionQuery::find_all_with_linked_by_user_id(&db, user.id).await {
                    // FIXME: This function assumes that ambition_id and objective_id are sorted.
                    Ok(ambitions) => {
                        let mut res: Vec<AmbitionVisibleWithLinks> = vec![];
                        for ambition in ambitions {
                            if res.len() > 0
                                && res.last().unwrap().id == ambition.id
                                && ambition.objective_id.is_some()
                            {
                                let mut last_ambition = res.pop().unwrap();
                                if last_ambition.objectives.last().unwrap().id != ambition.objective_id.unwrap() {
                                    last_ambition.push_objective(get_objective(&ambition));
                                }
                                if ambition.action_id.is_some() {
                                    last_ambition.push_action(get_action(&ambition));
                                }
                                res.push(last_ambition);
                            } else {
                                let mut res_ambition = AmbitionVisibleWithLinks {
                                    id: ambition.id,
                                    name: ambition.name.clone(),
                                    description: ambition.description.clone(),
                                    created_at: ambition.created_at,
                                    updated_at: ambition.updated_at,
                                    objectives: vec![],
                                };
                                if ambition.objective_id.is_some() {
                                    res_ambition.push_objective(get_objective(&ambition));
                                }
                                if ambition.action_id.is_some() {
                                    res_ambition.push_action(get_action(&ambition));
                                }
                                res.push(res_ambition);
                            }
                        }
                        HttpResponse::Ok().json(res)
                    }
                    Err(e) => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                        HttpResponse::InternalServerError().json(types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                }
            } else {
                match AmbitionQuery::find_all_by_user_id(&db, user.id).await {
                    Ok(ambitions) => HttpResponse::Ok().json(ambitions),
                    Err(e) => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                        HttpResponse::InternalServerError().json(types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                }
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}

fn get_objective(ambition: &AmbitionWithLinksQueryResult) -> ObjectiveVisibleWithActions {
    ObjectiveVisibleWithActions {
        id: ambition.objective_id.unwrap(),
        name: ambition.objective_name.clone().unwrap(),
        created_at: ambition.objective_created_at.unwrap(),
        updated_at: ambition.objective_updated_at.unwrap(),
        actions: vec![],
    }
}

fn get_action(ambition: &AmbitionWithLinksQueryResult) -> ActionVisible {
    ActionVisible {
        id: ambition.action_id.unwrap(),
        name: ambition.action_name.clone().unwrap(),
        created_at: ambition.action_created_at.unwrap(),
        updated_at: ambition.action_updated_at.unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, DbErr, Set};
    use types::{AmbitionVisible, AmbitionVisibleWithLinks};

    use crate::{
        entities::{ambitions_objectives, objectives_actions},
        test_utils,
    };

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_ambitions))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (ambition_1, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "ambition_for_get_1".to_string(),
            None,
            user.id,
        )
        .await?;
        let (ambition_2, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "ambition_for_get_2".to_string(),
            Some("ambition_for_get_2".to_string()),
            user.id,
        )
        .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_ambitions: Vec<AmbitionVisible> = test::read_body_json(resp).await;
        assert_eq!(
            serde_json::to_value(&returned_ambitions[0]).unwrap(),
            serde_json::json!({
                "id": ambition_1.id,
                "name": ambition_1.name,
                "description": ambition_1.description,
                "created_at": ambition_1.created_at,
                "updated_at": ambition_1.updated_at,
            })
        );
        assert_eq!(
            serde_json::to_value(&returned_ambitions[1]).unwrap(),
            serde_json::json!({
                "id": ambition_2.id,
                "name": ambition_2.name,
                "description": ambition_2.description,
                "created_at": ambition_2.created_at,
                "updated_at": ambition_2.updated_at,
            })
        );

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_with_links() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (ambition_1, objective_1, action_1) =
            test_utils::seed::create_set_of_ambition_objective_action(&db, user.id, true, true)
                .await?;
        let (ambition_2, objective_2, action_2) =
            test_utils::seed::create_set_of_ambition_objective_action(&db, user.id, false, false)
                .await?;
        let _ = objectives_actions::ActiveModel {
            objective_id: Set(objective_1.id),
            action_id: Set(action_2.id),
        }
        .insert(&db)
        .await?;
        let _ = ambitions_objectives::ActiveModel {
            ambition_id: Set(ambition_1.id),
            objective_id: Set(objective_2.id),
        }
        .insert(&db)
        .await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<AmbitionVisibleWithLinks> = test::read_body_json(resp).await;
        // Ambition_1
        assert_eq!(body[0].id, ambition_1.id);
        assert_eq!(body[0].name, ambition_1.name);
        assert_eq!(body[0].description, ambition_1.description);
        assert_eq!(body[0].created_at, ambition_1.created_at);
        assert_eq!(body[0].updated_at, ambition_1.updated_at);
        // Ambition_1-Objective_1
        assert_eq!(body[0].objectives.len(), 2);
        assert_eq!(
            serde_json::to_value(&body[0].objectives[0]).unwrap(),
            serde_json::json!({
                "id": objective_1.id,
                "name": objective_1.name,
                "created_at": objective_1.created_at,
                "updated_at": objective_1.updated_at,
                "actions": [
                    {
                        "id": action_1.id,
                        "name": action_1.name,
                        "created_at": action_1.created_at,
                        "updated_at": action_1.updated_at,
                    },
                    {
                        "id": action_2.id,
                        "name": action_2.name,
                        "created_at": action_2.created_at,
                        "updated_at": action_2.updated_at,
                    },
                ],
            })
        );
        assert_eq!(body[0].objectives[0].id, objective_1.id);
        assert_eq!(body[0].objectives[0].name, objective_1.name);
        assert_eq!(body[0].objectives[0].created_at, objective_1.created_at);
        assert_eq!(body[0].objectives[0].updated_at, objective_1.updated_at);
        assert_eq!(body[0].objectives[0].actions.len(), 2);
        assert_eq!(body[0].objectives[0].actions[0].id, action_1.id);
        assert_eq!(body[0].objectives[0].actions[0].name, action_1.name);
        assert_eq!(
            body[0].objectives[0].actions[0].created_at,
            action_1.created_at
        );
        assert_eq!(
            body[0].objectives[0].actions[0].updated_at,
            action_1.updated_at
        );
        assert_eq!(body[0].objectives[0].actions[1].id, action_2.id);
        assert_eq!(body[0].objectives[0].actions[1].name, action_2.name);
        assert_eq!(
            body[0].objectives[0].actions[1].created_at,
            action_2.created_at
        );
        assert_eq!(
            body[0].objectives[0].actions[1].updated_at,
            action_2.updated_at
        );
        // Ambition_1-Objective_2
        assert_eq!(
            serde_json::to_value(&body[0].objectives[1]).unwrap(),
            serde_json::json!({
                "id": objective_2.id,
                "name": objective_2.name,
                "created_at": objective_2.created_at,
                "updated_at": objective_2.updated_at,
                "actions": [],
            })
        );
        assert_eq!(body[0].objectives[1].actions.len(), 0);
        assert_eq!(body[0].objectives[1].id, objective_2.id);
        assert_eq!(body[0].objectives[1].name, objective_2.name);
        assert_eq!(body[0].objectives[1].created_at, objective_2.created_at);
        assert_eq!(body[0].objectives[1].updated_at, objective_2.updated_at);

        // Ambition_2
        assert_eq!(body[1].id, ambition_2.id);
        assert_eq!(body[1].name, ambition_2.name);
        assert_eq!(body[1].description, ambition_2.description);
        assert_eq!(body[1].created_at, ambition_2.created_at);
        assert_eq!(body[1].updated_at, ambition_2.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
