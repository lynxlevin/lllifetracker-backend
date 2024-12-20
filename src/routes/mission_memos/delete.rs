use crate::{
    entities::user as user_entity,
    services::mission_memo_mutation::MissionMemoMutation,
    types::{self, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    mission_memo_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a mission memo", skip(db, user, path_param))]
#[delete("/{mission_memo_id}")]
pub async fn delete_mission_memo(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MissionMemoMutation::delete(&db, path_param.mission_memo_id, user.id).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
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
    use sea_orm::{entity::prelude::*, ActiveValue::Set, DbErr, EntityTrait};

    use crate::{
        entities::{mission_memo, mission_memos_tags},
        test_utils,
    };

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(delete_mission_memo)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission Memo to delete.".to_string(),
            user.id,
        )
        .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        mission_memos_tags::ActiveModel {
            mission_memo_id: Set(mission_memo.id),
            tag_id: Set(ambition_tag.id),
        }
        .insert(&db)
        .await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", mission_memo.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let mission_memo_in_db = mission_memo::Entity::find_by_id(mission_memo.id)
            .one(&db)
            .await?;
        assert!(mission_memo_in_db.is_none());

        let mission_memos_tags_in_db = mission_memos_tags::Entity::find()
            .filter(mission_memos_tags::Column::MissionMemoId.eq(mission_memo.id))
            .filter(mission_memos_tags::Column::TagId.eq(ambition_tag.id))
            .one(&db)
            .await?;
        assert!(mission_memos_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission Memo to delete.".to_string(),
            user.id,
        )
        .await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", mission_memo.id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
