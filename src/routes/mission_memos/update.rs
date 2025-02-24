use entities::user as user_entity;
use ::types::{self, CustomDbErr, MissionMemoVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use crate::{
    services::mission_memo_mutation::{MissionMemoMutation, UpdateMissionMemo},
};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr, TransactionError};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    mission_memo_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    title: Option<String>,
    text: Option<String>,
    date: Option<chrono::NaiveDate>,
    tag_ids: Option<Vec<uuid::Uuid>>,
}

#[tracing::instrument(name = "Updating a mission memo", skip(db, user, req, path_param))]
#[put("/{mission_memo_id}")]
pub async fn update_mission_memo(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let form = UpdateMissionMemo {
                id: path_param.mission_memo_id,
                title: req.title.clone(),
                text: req.text.clone(),
                date: req.date,
                tag_ids: req.tag_ids.clone(),
                user_id: user.id,
            };
            match MissionMemoMutation::partial_update(&db, form).await {
                Ok(mission_memo) => {
                    let res: MissionMemoVisible = mission_memo.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => match e {
                    TransactionError::Transaction(DbErr::Custom(message)) => {
                        match message.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "Mission Memo with this id was not found".to_string(),
                                })
                            }
                        }
                    }
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
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait, QuerySelect};

    use entities::{mission_memo, mission_memos_tags};
    use test_utils::{self, *};

    use super::*;

    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    enum QueryAs {
        TagId,
    }

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(update_mission_memo)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let mission_memo = factory::mission_memo(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

        let form = RequestBody {
            title: Some("mission memo after update title".to_string()),
            text: Some("mission memo after update text".to_string()),
            date: Some(chrono::NaiveDate::from_ymd_opt(2024, 11, 3).unwrap()),
            tag_ids: Some(vec![ambition_tag.id]),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/{}", mission_memo.id))
            .set_json(&form)
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_mission_memo: MissionMemoVisible = test::read_body_json(res).await;
        assert_eq!(returned_mission_memo.title, form.title.clone().unwrap());
        assert_eq!(returned_mission_memo.text, form.text.clone().unwrap());
        assert_eq!(returned_mission_memo.date, form.date.unwrap());
        assert_eq!(returned_mission_memo.archived, mission_memo.archived);
        assert_eq!(returned_mission_memo.accomplished_at, mission_memo.accomplished_at);
        assert_eq!(returned_mission_memo.created_at, mission_memo.created_at);
        assert!(returned_mission_memo.updated_at > mission_memo.updated_at);

        let updated_mission_memo = mission_memo::Entity::find_by_id(returned_mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_mission_memo.title, form.title.unwrap());
        assert_eq!(updated_mission_memo.text, form.text.unwrap());
        assert_eq!(updated_mission_memo.date, form.date.unwrap());
        assert_eq!(updated_mission_memo.archived, mission_memo.archived);
        assert_eq!(updated_mission_memo.accomplished_at, mission_memo.accomplished_at);
        assert_eq!(updated_mission_memo.user_id, user.id);
        assert_eq!(
            updated_mission_memo.created_at,
            returned_mission_memo.created_at
        );
        assert_eq!(
            updated_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = mission_memos_tags::Entity::find()
            .column_as(mission_memos_tags::Column::TagId, QueryAs::TagId)
            .filter(mission_memos_tags::Column::MissionMemoId.eq(returned_mission_memo.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 1);
        assert!(linked_tag_ids.contains(&ambition_tag.id));

        Ok(())
    }

    #[actix_web::test]
    async fn not_found_if_invalid_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", uuid::Uuid::new_v4()))
            .set_json(RequestBody {
                title: None,
                text: None,
                date: None,
                tag_ids: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let mission_memo = factory::mission_memo(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", mission_memo.id))
            .set_json(RequestBody {
                title: None,
                text: None,
                date: None,
                tag_ids: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
