use crate::{
    entities::user as user_entity,
    services::memo_mutation::{MemoMutation, NewMemo},
    types::{self, MemoVisible, INTERNAL_SERVER_ERROR_MESSAGE},
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

#[tracing::instrument(name = "Creating a memo", skip(db, user))]
#[post("")]
pub async fn create_memo(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MemoMutation::create(
                &db,
                NewMemo {
                    title: req.title.clone(),
                    text: req.text.clone(),
                    date: req.date,
                    tag_ids: req.tag_ids.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(memo) => HttpResponse::Created().json(MemoVisible {
                    id: memo.id,
                    title: memo.title,
                    text: memo.text,
                    date: memo.date,
                    created_at: memo.created_at,
                    updated_at: memo.updated_at,
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
        entities::{memo, memos_tags},
        test_utils,
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
                .service(scope("/").service(create_memo))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (_, tag_0) =
            test_utils::seed::create_action_and_tag(&db, "action_0".to_string(), user.id).await?;
        let (_, tag_1) =
            test_utils::seed::create_action_and_tag(&db, "action_1".to_string(), user.id).await?;

        let memo_title = "New Memo".to_string();
        let memo_text = "This is a new memo for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                title: memo_title.clone(),
                text: memo_text.clone(),
                date: today,
                tag_ids: vec![tag_0.id, tag_1.id],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let returned_memo: MemoVisible = test::read_body_json(res).await;
        assert_eq!(returned_memo.title, memo_title.clone());
        assert_eq!(returned_memo.text, memo_text.clone());
        assert_eq!(returned_memo.date, today);

        let created_memo = memo::Entity::find_by_id(returned_memo.id)
            .filter(memo::Column::Title.eq(memo_title.clone()))
            .filter(memo::Column::Text.eq(memo_text.clone()))
            .filter(memo::Column::Date.eq(today))
            .filter(memo::Column::UserId.eq(user.id))
            .filter(memo::Column::CreatedAt.eq(returned_memo.created_at))
            .filter(memo::Column::UpdatedAt.eq(returned_memo.updated_at))
            .one(&db)
            .await?;
        assert!(created_memo.is_some());

        let linked_tag_ids: Vec<uuid::Uuid> = memos_tags::Entity::find()
            .column_as(memos_tags::Column::TagId, QueryAs::TagId)
            .filter(memos_tags::Column::MemoId.eq(returned_memo.id))
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
                title: "New Memo".to_string(),
                text: "This is a new memo for testing create method.".to_string(),
                date: chrono::Utc::now().date_naive(),
                tag_ids: vec![],
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
