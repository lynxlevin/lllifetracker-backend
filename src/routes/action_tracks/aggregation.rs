use crate::{
    services::action_track_query::ActionTrackQuery,
    types::{
        self, ActionTrackAggregation, ActionTrackAggregationDuration, INTERNAL_SERVER_ERROR_MESSAGE,
    },
};
use entities::user as user_entity;
use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct QueryParam {
    started_at_gte: Option<chrono::DateTime<chrono::FixedOffset>>,
    started_at_lte: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[tracing::instrument(name = "Aggregating a user's action tracks", skip(db, user))]
#[get("/aggregation")]
pub async fn aggregate_action_tracks(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let mut filters = ActionTrackQuery::get_default_filters();
            filters.started_at_gte = query.started_at_gte;
            filters.started_at_lte = query.started_at_lte;
            filters.inactive_only = true;
            match ActionTrackQuery::find_by_user_id_with_filters(&db, user.id, filters).await {
                Ok(action_tracks) => {
                    let mut res: Vec<ActionTrackAggregationDuration> = vec![];
                    for action_track in action_tracks {
                        if res.is_empty()
                            || res.last().unwrap().action_id != action_track.action_id.unwrap()
                        {
                            res.push(ActionTrackAggregationDuration {
                                action_id: action_track.action_id.unwrap(),
                                duration: action_track.duration.unwrap_or(0),
                            });
                        } else {
                            res.last_mut().unwrap().duration += action_track.duration.unwrap_or(0)
                        }
                    }
                    HttpResponse::Ok().json(ActionTrackAggregation {
                        durations_by_action: res,
                    })
                }
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
        http, test, App, HttpMessage,
    };
    use chrono::{DateTime, Duration, FixedOffset};
    use sea_orm::{entity::prelude::*, DbErr};

    use crate::test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(aggregate_action_tracks)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let query_started_at_gte: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-01-27T00:00:00Z").unwrap();
        let query_started_at_lte: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339("2025-01-27T23:59:59Z").unwrap();
        let action_0 = factory::action(user.id).insert(&db).await?;
        let _action_0_track_0 = factory::action_track(user.id)
            .started_at(query_started_at_gte - Duration::seconds(1))
            .duration(Some(120))
            .action_id(Some(action_0.id))
            .insert(&db)
            .await?;
        let action_0_track_1 = factory::action_track(user.id)
            .started_at(query_started_at_gte)
            .duration(Some(180))
            .action_id(Some(action_0.id))
            .insert(&db)
            .await?;
        let action_0_track_2 = factory::action_track(user.id)
            .started_at(query_started_at_lte)
            .duration(Some(350))
            .action_id(Some(action_0.id))
            .insert(&db)
            .await?;
        let _action_0_track_3 = factory::action_track(user.id)
            .started_at(query_started_at_lte + Duration::seconds(1))
            .duration(Some(550))
            .action_id(Some(action_0.id))
            .insert(&db)
            .await?;

        let req = test::TestRequest::get()
            .uri(&*format!(
                "/aggregation?started_at_gte={}&started_at_lte={}",
                query_started_at_gte.format("%Y-%m-%dT%H:%M:%SZ"),
                query_started_at_lte.format("%Y-%m-%dT%H:%M:%SZ")
            ))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        dbg!(&resp.response().body());
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_aggregation: ActionTrackAggregation = test::read_body_json(resp).await;

        let expected = ActionTrackAggregation {
            durations_by_action: vec![ActionTrackAggregationDuration {
                action_id: action_0.id,
                duration: action_0_track_1.duration.unwrap() + action_0_track_2.duration.unwrap(),
            }],
        };

        assert_eq!(returned_aggregation, expected);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::get().uri("/aggregation").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
