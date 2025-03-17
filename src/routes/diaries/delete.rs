use entities::user as user_entity;
use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use services::diary_mutation::DiaryMutation;
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    diary_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a diary", skip(db, user, path_param))]
#[delete("/{diary_id}")]
pub async fn delete_diary(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DiaryMutation::delete(&db, path_param.diary_id, user.id).await {
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
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait};

    use entities::{diary, diaries_tags};
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(delete_diary).app_data(Data::new(db))).await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;
        let (_, tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_diary_tag(&db, diary.id, tag.id).await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", diary.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let diary_in_db = diary::Entity::find_by_id(diary.id).one(&db).await?;
        assert!(diary_in_db.is_none());

        let diaries_tags_in_db = diaries_tags::Entity::find()
            .filter(diaries_tags::Column::DiaryId.eq(diary.id))
            .filter(diaries_tags::Column::TagId.eq(tag.id))
            .one(&db)
            .await?;
        assert!(diaries_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let diary = factory::diary(user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", diary.id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
