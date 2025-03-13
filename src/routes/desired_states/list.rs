use entities::user as user_entity;
use ::types::{
    self, ActionVisibleForLinking, AmbitionVisible, DesiredStateVisibleWithLinks,
    DesiredStateWithLinksQueryResult, INTERNAL_SERVER_ERROR_MESSAGE,
};
use services::desired_state_query::DesiredStateQuery;
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

#[tracing::instrument(name = "Listing a user's desired_states", skip(db, user))]
#[get("")]
pub async fn list_desired_states(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            if query.links.unwrap_or(false) {
                match DesiredStateQuery::find_all_with_linked_by_user_id(&db, user.id).await {
                    Ok(desired_states) => {
                        let mut res: Vec<DesiredStateVisibleWithLinks> = vec![];
                        for desired_state in desired_states {
                            if res.is_empty() || res.last().unwrap().id != desired_state.id {
                                let mut res_desired_state = DesiredStateVisibleWithLinks {
                                    id: desired_state.id,
                                    name: desired_state.name.clone(),
                                    description: desired_state.description.clone(),
                                    created_at: desired_state.created_at,
                                    updated_at: desired_state.updated_at,
                                    ambitions: vec![],
                                    actions: vec![],
                                };
                                if let Some(ambition) = get_ambition(&desired_state) {
                                    res_desired_state.push_ambition(ambition);
                                }
                                if let Some(action) = get_action(&desired_state) {
                                    res_desired_state.push_action(action);
                                }
                                res.push(res_desired_state);
                            } else {
                                let last_desired_state = res.last_mut().unwrap();
                                if let Some(ambition) = get_ambition(&desired_state) {
                                    if !last_desired_state.ambitions.contains(&ambition) {
                                        last_desired_state.push_ambition(ambition);
                                    }
                                }
                                if let Some(action) = get_action(&desired_state) {
                                    if !last_desired_state.actions.contains(&action) {
                                        last_desired_state.push_action(action);
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
                match DesiredStateQuery::find_all_by_user_id(&db, user.id).await {
                    Ok(desired_states) => HttpResponse::Ok().json(desired_states),
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

fn get_ambition(desired_state: &DesiredStateWithLinksQueryResult) -> Option<AmbitionVisible> {
    if desired_state.ambition_id.is_none() {
        return None;
    }
    Some(AmbitionVisible {
        id: desired_state.ambition_id.unwrap(),
        name: desired_state.ambition_name.clone().unwrap(),
        description: desired_state.ambition_description.clone(),
        created_at: desired_state.ambition_created_at.unwrap(),
        updated_at: desired_state.ambition_updated_at.unwrap(),
    })
}

fn get_action(desired_state: &DesiredStateWithLinksQueryResult) -> Option<ActionVisibleForLinking> {
    if desired_state.action_id.is_none() {
        return None;
    }
    Some(ActionVisibleForLinking {
        id: desired_state.action_id.unwrap(),
        name: desired_state.action_name.clone().unwrap(),
        description: desired_state.action_description.clone(),
        created_at: desired_state.action_created_at.unwrap(),
        updated_at: desired_state.action_updated_at.unwrap(),
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
    use ::types::DesiredStateVisible;

    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_desired_states))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let desired_state_0 = factory::desired_state(user.id)
            .name("desired_state_0".to_string())
            .insert(&db)
            .await?;
        let desired_state_1 = factory::desired_state(user.id)
            .name("desired_state_1".to_string())
            .insert(&db)
            .await?;
        let _archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_desired_states: Vec<DesiredStateVisible> = test::read_body_json(resp).await;
        assert_eq!(returned_desired_states[0].id, desired_state_0.id);
        assert_eq!(returned_desired_states[0].name, desired_state_0.name);
        assert_eq!(returned_desired_states[0].created_at, desired_state_0.created_at);
        assert_eq!(returned_desired_states[0].updated_at, desired_state_0.updated_at);

        assert_eq!(returned_desired_states[1].id, desired_state_1.id);
        assert_eq!(returned_desired_states[1].name, desired_state_1.name);
        assert_eq!(returned_desired_states[1].created_at, desired_state_1.created_at);
        assert_eq!(returned_desired_states[1].updated_at, desired_state_1.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_with_links() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let desired_state_0 = factory::desired_state(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let ambition_1 = factory::ambition(user.id).insert(&db).await?;
        let desired_state_1 = factory::desired_state(user.id).insert(&db).await?;
        let action_1 = factory::action(user.id).insert(&db).await?;
        factory::link_ambition_desired_state(&db, ambition_0.id, desired_state_0.id).await?;
        factory::link_ambition_desired_state(&db, ambition_1.id, desired_state_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_1.id).await?;
        factory::link_desired_state_action(&db, desired_state_1.id, action_1.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<DesiredStateVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);

        let mut expected_0 = serde_json::json!({
            "id": desired_state_0.id,
            "name": desired_state_0.name,
            "description": desired_state_0.description,
            "created_at": desired_state_0.created_at,
            "updated_at": desired_state_0.updated_at,
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
            "id": desired_state_1.id,
            "name": desired_state_1.name,
            "description": desired_state_1.description,
            "created_at": desired_state_1.created_at,
            "updated_at": desired_state_1.updated_at,
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
        let desired_state_0 = factory::desired_state(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let _archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_desired_state(&db, ambition_0.id, desired_state_0.id).await?;
        factory::link_ambition_desired_state(&db, archived_ambition.id, desired_state_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, archived_action.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<DesiredStateVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": desired_state_0.id,
            "name": desired_state_0.name,
            "description": desired_state_0.description,
            "created_at": desired_state_0.created_at,
            "updated_at": desired_state_0.updated_at,
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
        let desired_state = factory::desired_state(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_desired_state(&db, archived_ambition.id, desired_state.id).await?;
        factory::link_desired_state_action(&db, desired_state.id, archived_action.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<DesiredStateVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": desired_state.id,
            "name": desired_state.name,
            "description": desired_state.description,
            "created_at": desired_state.created_at,
            "updated_at": desired_state.updated_at,
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
