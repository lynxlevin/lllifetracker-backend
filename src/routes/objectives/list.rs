use crate::{
    entities::user as user_entity,
    services::objective_query::ObjectiveQuery,
    types::{
        self, ActionVisibleForLinking, AmbitionVisible, ObjectiveVisibleWithLinks,
        ObjectiveWithLinksQueryResult, INTERNAL_SERVER_ERROR_MESSAGE,
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

#[tracing::instrument(name = "Listing a user's objectives", skip(db, user))]
#[get("")]
pub async fn list_objectives(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            if query.links.unwrap_or(false) {
                match ObjectiveQuery::find_all_with_linked_by_user_id(&db, user.id).await {
                    Ok(objectives) => {
                        let mut res: Vec<ObjectiveVisibleWithLinks> = vec![];
                        for objective in objectives {
                            if res.is_empty() || res.last().unwrap().id != objective.id {
                                let mut res_objective = ObjectiveVisibleWithLinks {
                                    id: objective.id,
                                    name: objective.name.clone(),
                                    description: objective.description.clone(),
                                    created_at: objective.created_at,
                                    updated_at: objective.updated_at,
                                    ambitions: vec![],
                                    actions: vec![],
                                };
                                if let Some(ambition) = get_ambition(&objective) {
                                    res_objective.push_ambition(ambition);
                                }
                                if let Some(action) = get_action(&objective) {
                                    res_objective.push_action(action);
                                }
                                res.push(res_objective);
                            } else {
                                let last_objective = res.last_mut().unwrap();
                                if let Some(ambition) = get_ambition(&objective) {
                                    if !last_objective.ambitions.contains(&ambition) {
                                        last_objective.push_ambition(ambition);
                                    }
                                }
                                if let Some(action) = get_action(&objective) {
                                    if !last_objective.actions.contains(&action) {
                                        last_objective.push_action(action);
                                    }
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
                match ObjectiveQuery::find_all_by_user_id(&db, user.id).await {
                    Ok(objectives) => HttpResponse::Ok().json(objectives),
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

fn get_ambition(objective: &ObjectiveWithLinksQueryResult) -> Option<AmbitionVisible> {
    if objective.ambition_id.is_none() {
        return None;
    }
    Some(AmbitionVisible {
        id: objective.ambition_id.unwrap(),
        name: objective.ambition_name.clone().unwrap(),
        description: objective.ambition_description.clone(),
        created_at: objective.ambition_created_at.unwrap(),
        updated_at: objective.ambition_updated_at.unwrap(),
    })
}

fn get_action(objective: &ObjectiveWithLinksQueryResult) -> Option<ActionVisibleForLinking> {
    if objective.action_id.is_none() {
        return None;
    }
    Some(ActionVisibleForLinking {
        id: objective.action_id.unwrap(),
        name: objective.action_name.clone().unwrap(),
        description: objective.action_description.clone(),
        created_at: objective.action_created_at.unwrap(),
        updated_at: objective.action_updated_at.unwrap(),
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
    use types::ObjectiveVisible;

    use crate::test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_objectives))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let objective_0 = factory::objective(user.id)
            .name("objective_0".to_string())
            .insert(&db)
            .await?;
        let objective_1 = factory::objective(user.id)
            .name("objective_1".to_string())
            .insert(&db)
            .await?;
        let _archived_objective = factory::objective(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_objectives: Vec<ObjectiveVisible> = test::read_body_json(resp).await;
        assert_eq!(returned_objectives[0].id, objective_0.id);
        assert_eq!(returned_objectives[0].name, objective_0.name);
        assert_eq!(returned_objectives[0].created_at, objective_0.created_at);
        assert_eq!(returned_objectives[0].updated_at, objective_0.updated_at);

        assert_eq!(returned_objectives[1].id, objective_1.id);
        assert_eq!(returned_objectives[1].name, objective_1.name);
        assert_eq!(returned_objectives[1].created_at, objective_1.created_at);
        assert_eq!(returned_objectives[1].updated_at, objective_1.updated_at);

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
        let ambition_1 = factory::ambition(user.id).insert(&db).await?;
        let objective_1 = factory::objective(user.id).insert(&db).await?;
        let action_1 = factory::action(user.id).insert(&db).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_0.id).await?;
        factory::link_ambition_objective(&db, ambition_1.id, objective_0.id).await?;
        factory::link_objective_action(&db, objective_0.id, action_0.id).await?;
        factory::link_objective_action(&db, objective_0.id, action_1.id).await?;
        factory::link_objective_action(&db, objective_1.id, action_1.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<ObjectiveVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);

        let mut expected_0 = serde_json::json!({
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
            "actions": [
                {
                    "id": action_0.id,
                    "name": action_0.name,
                    "description": action_0.description,
                    "created_at": action_0.created_at,
                    "updated_at": action_0.updated_at,
                },
                {
                    "id": action_1.id,
                    "name": action_1.name,
                    "description": action_1.description,
                    "created_at": action_1.created_at,
                    "updated_at": action_1.updated_at,
                },
            ],
        });
        let expected_0_ambitions = expected_0["ambitions"].take();
        let expected_0_actions = expected_0["actions"].take();

        let mut body_0 = serde_json::to_value(&body[0]).unwrap();
        let body_0_ambitions = body_0["ambitions"].take();
        let body_0_actions = body_0["actions"].take();
        assert_eq!(expected_0, body_0);
        assert_eq!(expected_0_ambitions, body_0_ambitions);
        assert_eq!(expected_0_actions, body_0_actions);

        let expected_1 = serde_json::json!({
            "id": objective_1.id,
            "name": objective_1.name,
            "description": objective_1.description,
            "created_at": objective_1.created_at,
            "updated_at": objective_1.updated_at,
            "ambitions": [],
            "actions": [
                {
                    "id": action_1.id,
                    "name": action_1.name,
                    "description": action_1.description,
                    "created_at": action_1.created_at,
                    "updated_at": action_1.updated_at,
                }
            ],
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
        let _archived_objective = factory::objective(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_0.id).await?;
        factory::link_ambition_objective(&db, archived_ambition.id, objective_0.id).await?;
        factory::link_objective_action(&db, objective_0.id, action_0.id).await?;
        factory::link_objective_action(&db, objective_0.id, archived_action.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<ObjectiveVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
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
            "actions": [
                {
                    "id": action_0.id,
                    "name": action_0.name,
                    "description": action_0.description,
                    "created_at": action_0.created_at,
                    "updated_at": action_0.updated_at,
                },
            ],
        }]);

        let body = serde_json::to_value(&body).unwrap();
        assert_eq!(expected, body);

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_with_links_item_linked_to_archived_items_should_be_returned(
    ) -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let objective = factory::objective(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_objective(&db, archived_ambition.id, objective.id).await?;
        factory::link_objective_action(&db, objective.id, archived_action.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<ObjectiveVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": objective.id,
            "name": objective.name,
            "description": objective.description,
            "created_at": objective.created_at,
            "updated_at": objective.updated_at,
            "ambitions": [],
            "actions": [],
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
