use crate::{
    entities::user as user_entity,
    services::mission_memo_mutation::{MissionMemoMutation, NewMissionMemo},
    types::{self, MissionMemoVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    title: String,
    text: String,
    date: chrono::NaiveDate,
    tag_ids: Vec<uuid::Uuid>,
}

#[tracing::instrument(name = "Creating a mission memo", skip(db, user))]
#[post("")]
pub async fn create_mission_memo(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MissionMemoMutation::create(
                &db,
                NewMissionMemo {
                    title: req.title.clone(),
                    text: req.text.clone(),
                    date: req.date,
                    tag_ids: req.tag_ids.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(mission_memo) => HttpResponse::Created().json(MissionMemoVisible {
                    id: mission_memo.id,
                    title: mission_memo.title,
                    text: mission_memo.text,
                    date: mission_memo.date,
                    archived: mission_memo.archived,
                    accomplished_at: mission_memo.accomplished_at,
                    created_at: mission_memo.created_at,
                    updated_at: mission_memo.updated_at,
                }),
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

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait, QuerySelect};

    use crate::{
        entities::{mission_memo, mission_memos_tags},
        test_utils::{self, *},
    };

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
                .service(scope("/").service(create_mission_memo))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (_, tag_0) = factory::action(user.id).name("action_0".to_string()).insert_with_tag(&db).await?;
        let (_, tag_1) = factory::action(user.id).name("action_1".to_string()).insert_with_tag(&db).await?;

        let mission_memo_title = "New Mission Memo".to_string();
        let mission_memo_text = "This is a new mission memo for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                title: mission_memo_title.clone(),
                text: mission_memo_text.clone(),
                date: today,
                tag_ids: vec![tag_0.id, tag_1.id],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_mission_memo: MissionMemoVisible = test::read_body_json(res).await;
        assert_eq!(returned_mission_memo.title, mission_memo_title.clone());
        assert_eq!(returned_mission_memo.text, mission_memo_text.clone());
        assert_eq!(returned_mission_memo.date, today);
        assert_eq!(returned_mission_memo.archived, false);
        assert_eq!(returned_mission_memo.accomplished_at, None);

        let created_mission_memo = mission_memo::Entity::find_by_id(returned_mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_mission_memo.title, mission_memo_title.clone());
        assert_eq!(created_mission_memo.text, mission_memo_text.clone());
        assert_eq!(created_mission_memo.date, today);
        assert_eq!(created_mission_memo.archived, false);
        assert_eq!(created_mission_memo.accomplished_at, None);
        assert_eq!(created_mission_memo.user_id, user.id);
        assert_eq!(
            created_mission_memo.created_at,
            returned_mission_memo.created_at
        );
        assert_eq!(
            created_mission_memo.updated_at,
            returned_mission_memo.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = mission_memos_tags::Entity::find()
            .column_as(mission_memos_tags::Column::TagId, QueryAs::TagId)
            .filter(mission_memos_tags::Column::MissionMemoId.eq(returned_mission_memo.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 2);
        assert!(linked_tag_ids.contains(&tag_0.id));
        assert!(linked_tag_ids.contains(&tag_1.id));

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                title: "New MissionMemo".to_string(),
                text: "This is a new mission memo for testing create method.".to_string(),
                date: chrono::Utc::now().date_naive(),
                tag_ids: vec![],
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
