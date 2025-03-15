use entities::user as user_entity;
use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use services::desired_state_mutation::DesiredStateMutation;
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
///     => So no need for any handling when deleting or archiving an desired_state.
/// - Ordering numbers can be larger than the number of desired_states.
/// - Ordering number can be null, and null desired_states will be sorted last.
/// - Duplicate ordering numbers are OK, although there’s no knowing how those desired_states will be sorted, it only happens when un-archiving an desired_state.
///     => So it does not perplex the user.
///
/// Frontend takes care of that, because it’s simpler that way.
/// No need for handling ordering when creating, updating, archiving, un-archiving and deleting an desired_state.
/// Ordering numbers need only be updated on this endpoint.

#[tracing::instrument(name = "Bulk updating desired_state ordering", skip(db, user, req))]
#[put("/bulk_update_ordering")]
pub async fn bulk_update_desired_state_ordering(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateMutation::bulk_update_ordering(&db, user.id, req.ordering.clone()).await {
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

    use entities::desired_state;
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(bulk_update_desired_state_ordering)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let desired_state_0 = factory::desired_state(user.id).insert(&db).await?;
        let desired_state_1 = factory::desired_state(user.id).insert(&db).await?;
        let desired_state_2 = factory::desired_state(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri("/bulk_update_ordering")
            .set_json(RequestBody {
                ordering: vec![desired_state_0.id, desired_state_1.id],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let actin_in_db_0 = desired_state::Entity::find_by_id(desired_state_0.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(actin_in_db_0.ordering, Some(1));

        let actin_in_db_1 = desired_state::Entity::find_by_id(desired_state_1.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(actin_in_db_1.ordering, Some(2));

        let desired_state_in_db_2 = desired_state::Entity::find_by_id(desired_state_2.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db_2.ordering, None);

        Ok(())
    }
}
