use ::types::{
    self, DiaryVisibleWithTags, DiaryWithTagQueryResult, TagType, TagVisible,
    INTERNAL_SERVER_ERROR_MESSAGE,
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::diary_query::DiaryQuery;

#[tracing::instrument(name = "Listing user's diaries.", skip(db, user))]
#[get("")]
pub async fn list_diaries(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DiaryQuery::find_all_with_tags_by_user_id(&db, user.id).await {
                Ok(diaries) => {
                    let mut res: Vec<DiaryVisibleWithTags> = vec![];
                    for diary in diaries {
                        if is_first_diary_to_process(&res, &diary) {
                            let mut res_diary = DiaryVisibleWithTags {
                                id: diary.id,
                                text: diary.text.clone(),
                                date: diary.date,
                                score: diary.score,
                                tags: vec![],
                            };
                            if let Some(tag) = get_tag(&diary) {
                                res_diary.push_tag(tag);
                            }
                            res.push(res_diary);
                        } else {
                            if let Some(tag) = get_tag(&diary) {
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

fn is_first_diary_to_process(
    res: &Vec<DiaryVisibleWithTags>,
    diary: &DiaryWithTagQueryResult,
) -> bool {
    res.is_empty() || res.last().unwrap().id != diary.id
}

fn get_tag(diary: &DiaryWithTagQueryResult) -> Option<TagVisible> {
    if diary.tag_id.is_none() {
        return None;
    }

    if let Some(name) = diary.tag_ambition_name.clone() {
        Some(TagVisible {
            id: diary.tag_id.unwrap(),
            name,
            tag_type: TagType::Ambition,
            created_at: diary.tag_created_at.unwrap(),
        })
    } else if let Some(name) = diary.tag_desired_state_name.clone() {
        Some(TagVisible {
            id: diary.tag_id.unwrap(),
            name,
            tag_type: TagType::DesiredState,
            created_at: diary.tag_created_at.unwrap(),
        })
    } else if let Some(name) = diary.tag_action_name.clone() {
        Some(TagVisible {
            id: diary.tag_id.unwrap(),
            name,
            tag_type: TagType::Action,
            created_at: diary.tag_created_at.unwrap(),
        })
    } else {
        unimplemented!("Tag without link to Ambition/DesiredState/Action is not implemented yet.");
    }
}

#[cfg(test)]
mod tests {
    use ::types::TagType;
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use chrono::{Duration, Utc};
    use sea_orm::{entity::prelude::ActiveModelTrait, DbErr};

    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(scope("/").service(list_diaries))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let now = Utc::now();
        let diary_0 = factory::diary(user.id).text(None).insert(&db).await?;
        let diary_1 = factory::diary(user.id)
            .date((now - Duration::days(1)).date_naive())
            .insert(&db)
            .await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (desired_state, desired_state_tag) =
            factory::desired_state(user.id).insert_with_tag(&db).await?;
        factory::link_diary_tag(&db, diary_0.id, ambition_tag.id).await?;
        factory::link_diary_tag(&db, diary_1.id, desired_state_tag.id).await?;
        factory::link_diary_tag(&db, diary_1.id, action_tag.id).await?;

        let req = test::TestRequest::get().uri("/").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body: Vec<DiaryVisibleWithTags> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);

        let expected_0 = serde_json::json!({
            "id": diary_0.id,
            "text": diary_0.text.clone(),
            "date": diary_0.date,
            "score": diary_0.score,
            "tags": [
                {
                    "id": ambition_tag.id,
                    "name": ambition.name,
                    "tag_type": TagType::Ambition,
                    "created_at": ambition_tag.created_at,
                },
            ],
        });
        let body_0 = serde_json::to_value(&body[0]).unwrap();
        assert_eq!(expected_0, body_0);

        let expected_1 = serde_json::json!({
            "id": diary_1.id,
            "text": diary_1.text.clone(),
            "date": diary_1.date,
            "score": diary_1.score.clone(),
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
