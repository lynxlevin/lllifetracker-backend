use crate::{
    entities::user as user_entity,
    services::action::Mutation as ActionMutation,
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
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an action", skip(db, user, path_param))]
#[delete("/{action_id}")]
pub async fn delete_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::delete(&db, path_param.action_id, user.id).await {
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
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait, Set, TransactionTrait};

    use crate::{
        entities::{action, tag},
        test_utils,
    };

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(App::new().service(delete_action).app_data(Data::new(db))).await
    }

    async fn init_seed_data(db: &DbConn, user_id: uuid::Uuid) -> (uuid::Uuid, uuid::Uuid) {
        // NOTE: This transaction is for avoiding fk_constraint violation.
        db.transaction::<_, (uuid::Uuid, uuid::Uuid), DbErr>(|txn| {
            Box::pin(async move {
                let action = action::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    name: Set("action_before_update".to_string()),
                    user_id: Set(user_id),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                let tag = tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    action_id: Set(Some(action.id)),
                    user_id: Set(user_id),
                    ..Default::default()
                }
                .insert(txn)
                .await?;
                Ok((action.id, tag.id))
            })
        })
        .await
        .unwrap()
    }

    #[actix_web::test]
    async fn test_happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::get_or_create_user(&db).await?;
        let (action_id, tag_id) = init_seed_data(&db, user.id).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", action_id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let action_in_db = action::Entity::find_by_id(action_id).one(&db).await?;
        assert!(action_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag_id).one(&db).await?;
        assert!(tag_in_db.is_none());

        test_utils::flush_actions(&db).await?;
        Ok(())
    }

    #[actix_web::test]
    async fn test_unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::get_or_create_user(&db).await?;
        let (action_id, _) = init_seed_data(&db, user.id).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", action_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        test_utils::flush_actions(&db).await?;
        Ok(())
    }
}
