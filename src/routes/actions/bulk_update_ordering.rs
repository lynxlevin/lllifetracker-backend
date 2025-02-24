use entities::user as user_entity;
use crate::{
    services::action_mutation::ActionMutation,
    types::{self, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    ordering: Vec<uuid::Uuid>,
}

/// Fuzzy Ordering Design Decision
/// Ordering doesn’t need to be correctly serialized in the backend
/// - Skipping some numbers is OK.
///     => So no need for any handling when deleting or archiving an action.
/// - Ordering numbers can be larger than the number of actions.
/// - Ordering number can be null, and null actions will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those actions will be sorted, it only happens when un-archiving an action.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an action.
/// Ordering numbers need only be updated on this endpoint.

#[tracing::instrument(name = "Bulk updating action ordering", skip(db, user, req))]
#[put("/bulk_update_ordering")]
pub async fn bulk_update_action_ordering(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::bulk_update_ordering(&db, user.id, req.ordering.clone()).await {
                Ok(_) => HttpResponse::Ok().finish(),
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

    use entities::action;
    use crate::{
        test_utils::{self, *},
    };

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(bulk_update_action_ordering)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let action_1 = factory::action(user.id).insert(&db).await?;
        let action_2 = factory::action(user.id).insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let another_users_action = factory::action(another_user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri("/bulk_update_ordering")
            .set_json(RequestBody {
                ordering: vec![action_0.id, action_1.id],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let actin_in_db_0 = action::Entity::find_by_id(action_0.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(actin_in_db_0.ordering, Some(1));

        let actin_in_db_1 = action::Entity::find_by_id(action_1.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(actin_in_db_1.ordering, Some(2));

        let action_in_db_2 = action::Entity::find_by_id(action_2.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(action_in_db_2.ordering, None);

        let another_users_action_in_db = action::Entity::find_by_id(another_users_action.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(another_users_action_in_db.ordering, None);

        Ok(())
    }
}
