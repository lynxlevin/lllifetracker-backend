use crate::{
    entities::user as user_entity,
    services::memo_query::MemoQuery,
    types::{
        self, MemoVisibleWithTags, MemoWithTagQueryResult, TagType, TagVisible,
        INTERNAL_SERVER_ERROR_MESSAGE,
    },
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing user's memos.", skip(db, user))]
#[get("")]
pub async fn list_memos(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MemoQuery::find_all_with_tags_by_user_id(&db, user.id).await {
                Ok(memos) => {
                    let mut res: Vec<MemoVisibleWithTags> = vec![];
                    for memo in memos {
                        if res.is_empty() || res.last().unwrap().id != memo.id {
                            let mut res_memo = MemoVisibleWithTags {
                                id: memo.id,
                                title: memo.title.clone(),
                                text: memo.text.clone(),
                                date: memo.date,
                                created_at: memo.created_at,
                                updated_at: memo.updated_at,
                                tags: vec![],
                            };
                            if let Some(tag) = get_tag(&memo) {
                                res_memo.push_tag(tag);
                            }
                            res.push(res_memo);
                        } else {
                            if let Some(tag) = get_tag(&memo) {
                                let mut last_memo = res.pop().unwrap();
                                last_memo.push_tag(tag);
                                res.push(last_memo);
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

fn get_tag(memo: &MemoWithTagQueryResult) -> Option<TagVisible> {
    if memo.tag_id.is_none() {
        return None;
    }

    if let Some(name) = memo.tag_ambition_name.clone() {
        Some(TagVisible {
            id: memo.tag_id.unwrap(),
            name,
            tag_type: TagType::Ambition,
            created_at: memo.tag_created_at.unwrap(),
        })
    } else if let Some(name) = memo.tag_objective_name.clone() {
        Some(TagVisible {
            id: memo.tag_id.unwrap(),
            name,
            tag_type: TagType::Objective,
            created_at: memo.tag_created_at.unwrap(),
        })
    } else if let Some(name) = memo.tag_action_name.clone() {
        Some(TagVisible {
            id: memo.tag_id.unwrap(),
            name,
            tag_type: TagType::Action,
            created_at: memo.tag_created_at.unwrap(),
        })
    } else {
        unimplemented!("Tag without link to Ambition/Objective/Action is not implemented yet.");
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
    use types::TagType;

    use crate::{
        entities::memos_tags,
        test_utils::{self, factory},
    };

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_memos))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo_0 = test_utils::seed::create_memo(&db, "memo_0".to_string(), user.id).await?;
        let memo_1 = test_utils::seed::create_memo(&db, "memo_1".to_string(), user.id).await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (objective, objective_tag) =
            test_utils::seed::create_objective_and_tag(&db, "objective".to_string(), None, user.id)
                .await?;
        memos_tags::ActiveModel {
            memo_id: Set(memo_0.id),
            tag_id: Set(ambition_tag.id),
        }
        .insert(&db)
        .await?;
        memos_tags::ActiveModel {
            memo_id: Set(memo_1.id),
            tag_id: Set(objective_tag.id),
        }
        .insert(&db)
        .await?;
        memos_tags::ActiveModel {
            memo_id: Set(memo_1.id),
            tag_id: Set(action_tag.id),
        }
        .insert(&db)
        .await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<MemoVisibleWithTags> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);

        let expected_0 = serde_json::json!({
            "id": memo_1.id,
            "title": memo_1.title.clone(),
            "text": memo_1.text.clone(),
            "date": memo_1.date,
            "created_at": memo_1.created_at,
            "updated_at": memo_1.updated_at,
            "tags": [
                {
                    "id": objective_tag.id,
                    "name": objective.name,
                    "tag_type": TagType::Objective,
                    "created_at": objective_tag.created_at,
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
            "id": memo_0.id,
            "title": memo_0.title.clone(),
            "text": memo_0.text.clone(),
            "date": memo_0.date,
            "created_at": memo_0.created_at,
            "updated_at": memo_0.updated_at,
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
