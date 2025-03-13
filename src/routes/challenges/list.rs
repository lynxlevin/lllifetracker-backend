use entities::user as user_entity;
use ::types::{
    self, ChallengeVisibleWithTags, ChallengeWithTagQueryResult, TagType, TagVisible,
    INTERNAL_SERVER_ERROR_MESSAGE,
};
use services::challenge_query::ChallengeQuery;
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing user's mission memos.", skip(db, user))]
#[get("")]
pub async fn list_challenges(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ChallengeQuery::find_all_with_tags_by_user_id(&db, user.id).await {
                Ok(challenges) => {
                    let mut res: Vec<ChallengeVisibleWithTags> = vec![];
                    for challenge in challenges {
                        if res.is_empty() || res.last().unwrap().id != challenge.id {
                            let mut res_challenge = ChallengeVisibleWithTags {
                                id: challenge.id,
                                title: challenge.title.clone(),
                                text: challenge.text.clone(),
                                date: challenge.date,
                                archived: challenge.archived,
                                accomplished_at: challenge.accomplished_at,
                                created_at: challenge.created_at,
                                updated_at: challenge.updated_at,
                                tags: vec![],
                            };
                            if let Some(tag) = get_tag(&challenge) {
                                res_challenge.push_tag(tag);
                            }
                            res.push(res_challenge);
                        } else {
                            if let Some(tag) = get_tag(&challenge) {
                                res.last_mut().unwrap().push_tag(tag);
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
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}

fn get_tag(challenge: &ChallengeWithTagQueryResult) -> Option<TagVisible> {
    if challenge.tag_id.is_none() {
        return None;
    }

    if let Some(name) = challenge.tag_ambition_name.clone() {
        Some(TagVisible {
            id: challenge.tag_id.unwrap(),
            name,
            tag_type: TagType::Ambition,
            created_at: challenge.tag_created_at.unwrap(),
        })
    } else if let Some(name) = challenge.tag_desired_state_name.clone() {
        Some(TagVisible {
            id: challenge.tag_id.unwrap(),
            name,
            tag_type: TagType::DesiredState,
            created_at: challenge.tag_created_at.unwrap(),
        })
    } else if let Some(name) = challenge.tag_action_name.clone() {
        Some(TagVisible {
            id: challenge.tag_id.unwrap(),
            name,
            tag_type: TagType::Action,
            created_at: challenge.tag_created_at.unwrap(),
        })
    } else {
        unimplemented!("Tag without link to Ambition/DesiredState/Action is not implemented yet.");
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
    use sea_orm::{entity::prelude::*, DbErr};
    use ::types::TagType;

    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_challenges))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let challenge_0 = factory::challenge(user.id)
            .title("challenge_0".to_string())
            .insert(&db)
            .await?;
        let challenge_1 = factory::challenge(user.id)
            .title("challenge_1".to_string())
            .insert(&db)
            .await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (desired_state, desired_state_tag) = factory::desired_state(user.id).insert_with_tag(&db).await?;
        factory::link_challenge_tag(&db, challenge_0.id, ambition_tag.id).await?;
        factory::link_challenge_tag(&db, challenge_1.id, desired_state_tag.id).await?;
        factory::link_challenge_tag(&db, challenge_1.id, action_tag.id).await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<ChallengeVisibleWithTags> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);

        let expected_0 = serde_json::json!({
            "id": challenge_1.id,
            "title": challenge_1.title.clone(),
            "text": challenge_1.text.clone(),
            "date": challenge_1.date,
            "archived": challenge_1.archived,
            "accomplished_at": challenge_1.accomplished_at,
            "created_at": challenge_1.created_at,
            "updated_at": challenge_1.updated_at,
            "tags": [
                {
                    "id": desired_state_tag.id,
                    "name": desired_state.name,
                    "tag_type": TagType::DesiredState,
                    "created_at": desired_state_tag.created_at,
                },
                {
                    "id": action_tag.id,
                    "name": action.name,
                    "tag_type": TagType::Action,
                    "created_at": action_tag.created_at,
                },
            ],
        });

        let body_0 = serde_json::to_value(&body[0]).unwrap();
        assert_eq!(expected_0, body_0);

        let expected_1 = serde_json::json!({
            "id": challenge_0.id,
            "title": challenge_0.title.clone(),
            "text": challenge_0.text.clone(),
            "date": challenge_0.date,
            "archived": challenge_0.archived,
            "accomplished_at": challenge_0.accomplished_at,
            "created_at": challenge_0.created_at,
            "updated_at": challenge_0.updated_at,
            "tags": [
                {
                    "id": ambition_tag.id,
                    "name": ambition.name,
                    "tag_type": TagType::Ambition,
                    "created_at": ambition_tag.created_at,
                },
            ],
        });
        let body_1 = serde_json::to_value(&body[1]).unwrap();
        assert_eq!(expected_1, body_1,);

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
