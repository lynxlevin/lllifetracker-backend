use std::vec;

use entities::user as user_entity;
use ::types::{
    self, ActionVisibleForLinking, AmbitionVisibleWithLinks, AmbitionWithLinksQueryResult, DesiredStateVisibleWithActions, INTERNAL_SERVER_ERROR_MESSAGE
};
use services::ambition_query::AmbitionQuery;
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
    show_archived_only: Option<bool>,
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
                    // FIXME: This function assumes that ambition_id and desired_state_id are sorted.
                    Ok(ambitions) => {
                        let mut res: Vec<AmbitionVisibleWithLinks> = vec![];
                        for ambition in ambitions {
                            if res.is_empty() || res.last().unwrap().id != ambition.id {
                                let mut res_ambition = AmbitionVisibleWithLinks {
                                    id: ambition.id,
                                    name: ambition.name.clone(),
                                    description: ambition.description.clone(),
                                    created_at: ambition.created_at,
                                    updated_at: ambition.updated_at,
                                    desired_states: vec![],
                                };
                                if let Some(desired_state) = get_desired_state(&ambition) {
                                    res_ambition.push_desired_state(desired_state);
                                    if let Some(action) = get_action(&ambition) {
                                        res_ambition.push_action(action);
                                    }
                                }
                                res.push(res_ambition);
                            } else {
                                if let Some(desired_state) = get_desired_state(&ambition) {
                                    let last_ambition = res.last_mut().unwrap();
                                    if desired_state.id != last_ambition.desired_states.last().unwrap().id {
                                        last_ambition.push_desired_state(desired_state);
                                    }
                                    if let Some(action) = get_action(&ambition) {
                                        last_ambition.push_action(action);
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
                match AmbitionQuery::find_all_by_user_id(&db, user.id, query.show_archived_only.unwrap_or(false)).await {
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

fn get_desired_state(ambition: &AmbitionWithLinksQueryResult) -> Option<DesiredStateVisibleWithActions> {
    if ambition.desired_state_id.is_none() {
        return None;
    }
    Some(DesiredStateVisibleWithActions {
        id: ambition.desired_state_id.unwrap(),
        name: ambition.desired_state_name.clone().unwrap(),
        description: ambition.desired_state_description.clone(),
        created_at: ambition.desired_state_created_at.unwrap(),
        updated_at: ambition.desired_state_updated_at.unwrap(),
        actions: vec![],
    })
}

fn get_action(ambition: &AmbitionWithLinksQueryResult) -> Option<ActionVisibleForLinking> {
    if ambition.action_id.is_none() {
        return None;
    }
    Some(ActionVisibleForLinking {
        id: ambition.action_id.unwrap(),
        name: ambition.action_name.clone().unwrap(),
        description: ambition.action_description.clone(),
        created_at: ambition.action_created_at.unwrap(),
        updated_at: ambition.action_updated_at.unwrap(),
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
    use ::types::{AmbitionVisible, AmbitionVisibleWithLinks};

    use test_utils::{self, *};

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
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id)
            .name("ambition_0".to_string())
            .insert(&db)
            .await?;
        let ambition_1 = factory::ambition(user.id)
            .name("ambition1".to_string())
            .description(Some("ambition1".to_string()))
            .insert(&db)
            .await?;
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_ambitions: Vec<AmbitionVisible> = test::read_body_json(resp).await;
        assert_eq!(returned_ambitions.len(), 2);
        assert_eq!(
            serde_json::to_value(&returned_ambitions[0]).unwrap(),
            serde_json::json!({
                "id": ambition_0.id,
                "name": ambition_0.name,
                "description": ambition_0.description,
                "created_at": ambition_0.created_at,
                "updated_at": ambition_0.updated_at,
            })
        );
        assert_eq!(
            serde_json::to_value(&returned_ambitions[1]).unwrap(),
            serde_json::json!({
                "id": ambition_1.id,
                "name": ambition_1.name,
                "description": ambition_1.description,
                "created_at": ambition_1.created_at,
                "updated_at": ambition_1.updated_at,
            })
        );

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
        factory::link_ambition_desired_state(&db, ambition_0.id, desired_state_1.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_1.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<AmbitionVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);

        let mut expected_0 = serde_json::json!({
            "id": ambition_0.id,
            "name": ambition_0.name,
            "description": ambition_0.description,
            "created_at": ambition_0.created_at,
            "updated_at": ambition_0.updated_at,
            "desired_states": [
                {
                    "id": desired_state_0.id,
                    "name": desired_state_0.name,
                    "description": desired_state_0.description,
                    "created_at": desired_state_0.created_at,
                    "updated_at": desired_state_0.updated_at,
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
                },
                {
                    "id": desired_state_1.id,
                    "name": desired_state_1.name,
                    "description": desired_state_1.description,
                    "created_at": desired_state_1.created_at,
                    "updated_at": desired_state_1.updated_at,
                    "actions": [],
                }
            ],
        });
        let expected_0_desired_states_0 = expected_0["desired_states"][0].take();
        let expected_0_desired_states_1 = expected_0["desired_states"][1].take();

        let mut body_0 = serde_json::to_value(&body[0]).unwrap();
        let body_0_desired_states_0 = body_0["desired_states"][0].take();
        let body_0_desired_states_1 = body_0["desired_states"][1].take();
        assert_eq!(expected_0, body_0,);
        assert_eq!(expected_0_desired_states_0, body_0_desired_states_0);
        assert_eq!(expected_0_desired_states_1, body_0_desired_states_1);

        let expected_1 = serde_json::json!({
            "id": ambition_1.id,
            "name": ambition_1.name,
            "description": ambition_1.description,
            "created_at": ambition_1.created_at,
            "updated_at": ambition_1.updated_at,
            "desired_states": [],
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
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_desired_state(&db, ambition_0.id, desired_state_0.id).await?;
        factory::link_ambition_desired_state(&db, ambition_0.id, archived_desired_state.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, archived_action.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<AmbitionVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": ambition_0.id,
            "name": ambition_0.name,
            "description": ambition_0.description,
            "created_at": ambition_0.created_at,
            "updated_at": ambition_0.updated_at,
            "desired_states": [
                {
                    "id": desired_state_0.id,
                    "name": desired_state_0.name,
                    "description": desired_state_0.description,
                    "created_at": desired_state_0.created_at,
                    "updated_at": desired_state_0.updated_at,
                    "actions": [
                        {
                            "id": action_0.id,
                            "name": action_0.name,
                            "description": action_0.description,
                            "created_at": action_0.created_at,
                            "updated_at": action_0.updated_at,
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
    async fn happy_path_with_links_item_linked_to_archived_items_should_be_returned(
    ) -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;
        let archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_desired_state(&db, ambition.id, archived_desired_state.id).await?;
        factory::link_desired_state_action(&db, archived_desired_state.id, archived_action.id).await?;

        let req = test::TestRequest::get().uri("/?links=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<AmbitionVisibleWithLinks> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": ambition.id,
            "name": ambition.name,
            "description": ambition.description,
            "created_at": ambition.created_at,
            "updated_at": ambition.updated_at,
            "desired_states": [],
        }]);

        let body = serde_json::to_value(&body).unwrap();
        assert_eq!(expected, body);

        Ok(())
    }

    #[actix_web::test]
    async fn happy_path_show_archived_only() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let _ambition = factory::ambition(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/?show_archived_only=true").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<AmbitionVisible> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);

        let expected = serde_json::json!([{
            "id": archived_ambition.id,
            "name": archived_ambition.name,
            "description": archived_ambition.description,
            "created_at": archived_ambition.created_at,
            "updated_at": archived_ambition.updated_at,
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
