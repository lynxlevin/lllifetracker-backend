use crate::{
    entities::user as user_entity,
    services::action_query::ActionQuery,
    types::{
        self, ActionVisibleWithLinks, ActionWithLinksQueryResult, AmbitionVisible,
        ObjectiveVisibleWithAmbitions, INTERNAL_SERVER_ERROR_MESSAGE,
    },
};
use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct QueryParam {
    links: Option<bool>,
}

#[tracing::instrument(name = "Listing a user's actions", skip(db, user))]
#[get("")]
pub async fn list_actions(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            // MYMEMO: Should return same type in both conditions, maybe separate into different endpoints.
            if query.links.unwrap_or(false) {
                match ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await {
                    Ok(actions) => {
                        let mut res: Vec<ActionVisibleWithLinks> = vec![];
                        for action in actions {
                            if res.is_empty() || res.last().unwrap().id != action.id {
                                let mut res_action = ActionVisibleWithLinks {
                                    id: action.id,
                                    name: action.name.clone(),
                                    description: action.description.clone(),
                                    created_at: action.created_at,
                                    updated_at: action.updated_at,
                                    objectives: vec![],
                                };
                                if let Some(objective) = get_objective(&action) {
                                    res_action.push_objective(objective);
                                    if let Some(ambition) = get_ambition(&action) {
                                        res_action.push_ambition(ambition);
                                    }
                                }
                                res.push(res_action);
                            } else {
                                if let Some(objective) = get_objective(&action) {
                                    let mut last_action = res.pop().unwrap();
                                    if objective.id != last_action.objectives.last().unwrap().id {
                                        last_action.push_objective(objective);
                                    }
                                    if let Some(ambition) = get_ambition(&action) {
                                        last_action.push_ambition(ambition);
                                    }
                                    res.push(last_action);
                                }
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
                match ActionQuery::find_all_by_user_id(&db, user.id).await {
                    Ok(actions) => HttpResponse::Ok().json(actions),
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

fn get_objective(action: &ActionWithLinksQueryResult) -> Option<ObjectiveVisibleWithAmbitions> {
    if action.objective_id.is_none() {
        return None;
    }
    Some(ObjectiveVisibleWithAmbitions {
        id: action.objective_id.unwrap(),
        name: action.objective_name.clone().unwrap(),
        description: action.objective_description.clone(),
        created_at: action.objective_created_at.unwrap(),
        updated_at: action.objective_updated_at.unwrap(),
        ambitions: vec![],
    })
}

fn get_ambition(action: &ActionWithLinksQueryResult) -> Option<AmbitionVisible> {
    if action.ambition_id.is_none() {
        return None;
    }
    Some(AmbitionVisible {
        id: action.ambition_id.unwrap(),
        name: action.ambition_name.clone().unwrap(),
        description: action.ambition_description.clone(),
        created_at: action.ambition_created_at.unwrap(),
        updated_at: action.ambition_updated_at.unwrap(),
    })
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
    use sea_orm::{entity::prelude::*, DbErr};
    use types::{ActionVisible, ActionVisibleWithLinks};

    use crate::test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_actions))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action_0 = factory::action(user.id)
            .name("action_0".to_string())
            .insert(&db)
            .await?;
        let action_1 = factory::action(user.id)
            .name("action_1".to_string())
            .insert(&db)
            .await?;
        let _archived_action = factory::action(user.id).archived(true).insert(&db).await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_actions: Vec<ActionVisible> = test::read_body_json(resp).await;
        assert_eq!(returned_actions.len(), 2);

        assert_eq!(returned_actions[0].id, action_0.id);
        assert_eq!(returned_actions[0].name, action_0.name);
        assert_eq!(returned_actions[0].description, action_0.description);
        assert_eq!(returned_actions[0].created_at, action_0.created_at);
        assert_eq!(returned_actions[0].updated_at, action_0.updated_at);

        assert_eq!(returned_actions[1].id, action_1.id);
        assert_eq!(returned_actions[1].name, action_1.name);
        assert_eq!(returned_actions[1].description, action_1.description);
        assert_eq!(returned_actions[1].created_at, action_1.created_at);
        assert_eq!(returned_actions[1].updated_at, action_1.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_with_links() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let objective_0 = factory::objective(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_0.id).await?;
        factory::link_objective_action(&db, objective_0.id, action_0.id).await?;
        let ambition_1 = factory::ambition(user.id).insert(&db).await?;
        let objective_1 = factory::objective(user.id).insert(&db).await?;
        let action_1 = factory::action(user.id).insert(&db).await?;
        factory::link_ambition_objective(&db, ambition_1.id, objective_0.id).await?;
        factory::link_objective_action(&db, objective_1.id, action_0.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<ActionVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);

        let mut expected_0 = serde_json::json!({
            "id": action_0.id,
            "name": action_0.name,
            "description": action_0.description,
            "created_at": action_0.created_at,
            "updated_at": action_0.updated_at,
            "objectives": [
                {
                    "id": objective_0.id,
                    "name": objective_0.name,
                    "description": objective_0.description,
                    "created_at": objective_0.created_at,
                    "updated_at": objective_0.updated_at,
                    "ambitions": [
                        {
                            "id": ambition_0.id,
                            "name": ambition_0.name,
                            "description": ambition_0.description,
                            "created_at": ambition_0.created_at,
                            "updated_at": ambition_0.updated_at,
                        },
                        {
                            "id": ambition_1.id,
                            "name": ambition_1.name,
                            "description": ambition_1.description,
                            "created_at": ambition_1.created_at,
                            "updated_at": ambition_1.updated_at,
                        },
                    ],
                },
                {
                    "id": objective_1.id,
                    "name": objective_1.name,
                    "description": objective_1.description,
                    "created_at": objective_1.created_at,
                    "updated_at": objective_1.updated_at,
                    "ambitions": [],
                }
            ],
        });
        let expected_0_objectives_0 = expected_0["objectives"][0].take();
        let expected_0_objectives_1 = expected_0["objectives"][1].take();

        let mut body_0 = serde_json::to_value(&body[0]).unwrap();
        let body_0_objectives_0 = body_0["objectives"][0].take();
        let body_0_objectives_1 = body_0["objectives"][1].take();
        assert_eq!(expected_0, body_0);
        assert_eq!(expected_0_objectives_0, body_0_objectives_0);
        assert_eq!(expected_0_objectives_1, body_0_objectives_1);

        let expected_1 = serde_json::json!({
            "id": action_1.id,
            "name": action_1.name,
            "description": action_1.description,
            "created_at": action_1.created_at,
            "updated_at": action_1.updated_at,
            "objectives": [],
        });
        let body_1 = serde_json::to_value(&body[1]).unwrap();
        assert_eq!(expected_1, body_1,);

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_with_links_archived_items_should_not_be_returned() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let objective_0 = factory::objective(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_objective = factory::objective(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let _archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_objective_action(&db, objective_0.id, action_0.id).await?;
        factory::link_objective_action(&db, archived_objective.id, action_0.id).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_0.id).await?;
        factory::link_ambition_objective(&db, archived_ambition.id, objective_0.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<ActionVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": action_0.id,
            "name": action_0.name,
            "description": action_0.description,
            "created_at": action_0.created_at,
            "updated_at": action_0.updated_at,
            "objectives": [
                {
                    "id": objective_0.id,
                    "name": objective_0.name,
                    "description": objective_0.description,
                    "created_at": objective_0.created_at,
                    "updated_at": objective_0.updated_at,
                    "ambitions": [
                        {
                            "id": ambition_0.id,
                            "name": ambition_0.name,
                            "description": ambition_0.description,
                            "created_at": ambition_0.created_at,
                            "updated_at": ambition_0.updated_at,
                        },
                    ],
                },
            ],
        }]);

        let body = serde_json::to_value(&body).unwrap();
        assert_eq!(expected, body);

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
