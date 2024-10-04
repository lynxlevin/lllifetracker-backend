use crate::{
    entities::user as user_entity,
    services::action::{Mutation as ActionMutation, NewAction},
    types::{self, ActionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    name: String,
}

#[tracing::instrument(name = "Creating an action", skip(db, user))]
#[post("")]
pub async fn create_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::create_with_tag(
                &db,
                NewAction {
                    name: req.name.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(action) => HttpResponse::Ok().json(ActionVisible {
                    id: action.id,
                    name: action.name,
                    created_at: action.created_at,
                    updated_at: action.updated_at,
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

// #[cfg(test)]
// mod tests {
//     use actix_web::{test, FromRequest};
//     use migration::{Migrator, MigratorTrait};
//     use sea_orm::{entity::prelude::*, DbConn, DbErr, EntityTrait};

//     use crate::{
//         entities::{action, tag, user},
//         startup::get_database_connection,
//     };

//     use super::*;

//     #[actix_web::test]
//     async fn main() -> Result<(), DbErr> {
//         dotenvy::from_filename(".env.test").unwrap();
//         let db = get_database_connection().await;
//         Migrator::up(&db, None).await.unwrap();

//         flush(&db).await?;

//         Ok(())
//     }

//     async fn flush(db: &DbConn) -> Result<(), DbErr> {
//         tag::Entity::delete_many().exec(db).await?;
//         action::Entity::delete_many().exec(db).await?;
//         Ok(())
//     }

//     async fn test_happy_path(db: &DbConn) -> Result<(), DbErr> {
//         let user = user::Entity::find()
//             .filter(user::Column::Email.eq("test@test.com".to_string()))
//             .one(db)
//             .await?
//             .unwrap();

//         let request_body = RequestBody {
//             name: "Test action".to_string(),
//         };
//         let req = test::TestRequest::default().app_data(AppState { conn: db, redis_pool: })

//         Ok(())
//     }
// }
