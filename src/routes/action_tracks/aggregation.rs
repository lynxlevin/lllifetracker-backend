// use crate::{
//     entities::user as user_entity,
//     services::action_track_query::ActionTrackQuery,
//     types::{
//         self, ActionTrackAggregation, ActionTrackAggregationDuration, INTERNAL_SERVER_ERROR_MESSAGE,
//     },
// };
// use actix_web::{
//     get,
//     web::{Data, Query, ReqData},
//     HttpResponse,
// };
// use sea_orm::DbConn;
// use serde::Deserialize;

// #[derive(Deserialize, Debug)]
// struct QueryParam {
//     started_at_from: Option<chrono::DateTime<chrono::FixedOffset>>,
//     started_at_to: Option<chrono::DateTime<chrono::FixedOffset>>,
// }

// #[tracing::instrument(name = "Aggregating a user's action tracks", skip(db, user))]
// #[get("/aggregate")]
// pub async fn aggregate_action_tracks(
//     db: Data<DbConn>,
//     user: Option<ReqData<user_entity::Model>>,
//     query: Query<QueryParam>,
// ) -> HttpResponse {
//     match user {
//         Some(user) => {
//             let user = user.into_inner();
//             match ActionTrackQuery::find_by_user_id(
//                 &db,
//                 user.id,
//                 query.active_only.unwrap_or(false),
//             )
//             .await
//             {
//                 Ok(action_tracks) => HttpResponse::Ok().json(action_tracks),
//                 Err(e) => {
//                     tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
//                     HttpResponse::InternalServerError().json(types::ErrorResponse {
//                         error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
//                     })
//                 }
//             }
//         }
//         None => HttpResponse::Unauthorized().json("You are not logged in."),
//     }
// }

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use actix_http::Request;
//     use actix_web::{
//         dev::{Service, ServiceResponse},
//         http, test, App, HttpMessage,
//     };
//     use chrono::{DateTime, Duration, FixedOffset};
//     use sea_orm::{entity::prelude::*, DbErr};

//     use crate::test_utils::{self, *};

//     use super::*;

//     async fn init_app(
//         db: DbConn,
//     ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
//         test::init_service(
//             App::new()
//                 .service(aggregate_action_tracks)
//                 .app_data(Data::new(db)),
//         )
//         .await
//     }

//     #[actix_web::test]
//     async fn happy_path() -> Result<(), DbErr> {
//         let db = test_utils::init_db().await?;
//         let app = init_app(db.clone()).await;
//         let user = factory::user().insert(&db).await?;
//         let query_started_at_from: DateTime<FixedOffset> =
//             DateTime::from_str("2025-01-27T00:00:00Z").unwrap();
//         let query_started_at_to: DateTime<FixedOffset> =
//             DateTime::from_str("2025-01-27T23:59:59Z").unwrap();
//         let action_0 = factory::action(user.id).insert(&db).await?;
//         let action_1 = factory::action(user.id).insert(&db).await?;
//         let _action_0_track_0 = factory::action_track(user.id)
//             .started_at(query_started_at_from - Duration::seconds(1))
//             .duration(Some(120))
//             .action_id(Some(action_0.id))
//             .insert(&db)
//             .await?;
//         let action_0_track_1 = factory::action_track(user.id)
//             .duration(Some(180))
//             .action_id(Some(action_0.id))
//             .insert(&db)
//             .await?;
//         let action_0_track_2 = factory::action_track(user.id)
//             .duration(Some(350))
//             .action_id(Some(action_0.id))
//             .insert(&db)
//             .await?;
//         let _action_0_track_3 = factory::action_track(user.id)
//             .started_at(query_started_at_to + Duration::seconds(1))
//             .duration(Some(550))
//             .action_id(Some(action_0.id))
//             .insert(&db)
//             .await?;

//         let req = test::TestRequest::get()
//             .uri(&*format!(
//                 "/aggregation?started_at_from={}&started_at_to={}",
//                 query_started_at_from, query_started_at_to
//             ))
//             .to_request();
//         req.extensions_mut().insert(user.clone());

//         let resp = test::call_service(&app, req).await;
//         assert_eq!(resp.status(), http::StatusCode::OK);

//         let returned_aggregation: ActionTrackAggregation = test::read_body_json(resp).await;

//         let expected = ActionTrackAggregation {
//             durations_by_action: vec![
//                 ActionTrackAggregationDuration {
//                     action_id: action_0.id,
//                     duration: action_0_track_1.duration.unwrap()
//                         + action_0_track_2.duration.unwrap(),
//                 },
//                 ActionTrackAggregationDuration {
//                     action_id: action_1.id,
//                     duration: 0,
//                 },
//             ],
//         };

//         assert_eq!(returned_aggregation, expected);

//         Ok(())
//     }

//     #[actix_web::test]
//     async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
//         let db = test_utils::init_db().await?;
//         let app = init_app(db.clone()).await;

//         let req = test::TestRequest::get().uri("/aggregation").to_request();

//         let resp = test::call_service(&app, req).await;
//         assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

//         Ok(())
//     }
// }
