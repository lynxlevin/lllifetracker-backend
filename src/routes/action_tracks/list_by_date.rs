use entities::{sea_orm_active_enums::TimezoneEnum, user as user_entity};
use crate::{
    services::action_track_query::ActionTrackQuery,
    types::{self, ActionTrackWithActionName, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use chrono::{FixedOffset, TimeZone};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing a user's action tracks by date", skip(db, user))]
#[get("/by_date")]
pub async fn list_action_tracks_by_date(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionTrackQuery::find_all_by_user_id(&db, user.id, false).await {
                Ok(action_tracks) => {
                    let mut res: Vec<Vec<ActionTrackWithActionName>> = vec![];
                    let user_offset = match user.timezone {
                        TimezoneEnum::Utc => FixedOffset::east_opt(0).unwrap(),
                        TimezoneEnum::AsiaTokyo => FixedOffset::east_opt(9 * 3600).unwrap(),
                    };
                    for action_track in action_tracks {
                        if res.is_empty()
                            || !started_on_same_day(
                                res.last().unwrap().last().unwrap(),
                                &action_track,
                                &user_offset,
                            )
                        {
                            res.push(vec![action_track])
                        } else {
                            res.last_mut().unwrap().push(action_track);
                        }
                    }
                    HttpResponse::Ok().json(res)
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

fn started_on_same_day<Tz2: TimeZone>(
    date_1: &ActionTrackWithActionName,
    date_2: &ActionTrackWithActionName,
    user_timezone: &Tz2,
) -> bool {
    date_1.started_at.with_timezone(user_timezone).date_naive()
        == date_2.started_at.with_timezone(user_timezone).date_naive()
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test, App, HttpMessage,
    };
    use chrono::{Duration, Utc};
    use sea_orm::{entity::prelude::*, DbErr};
    use types::ActionTrackWithActionName;

    use crate::test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(list_action_tracks_by_date)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let now = Utc::now();
        let action_track_0 = factory::action_track(user.id)
            .duration(Some(120))
            .insert(&db)
            .await?;
        let action_track_1 = factory::action_track(user.id)
            .duration(Some(120))
            .insert(&db)
            .await?;
        let action_track_2 = factory::action_track(user.id)
            .duration(Some(120))
            .started_at((now - Duration::days(1)).into())
            .action_id(Some(action.id))
            .insert(&db)
            .await?;

        let req = test::TestRequest::get().uri("/by_date").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let returned_action_tracks: Vec<Vec<ActionTrackWithActionName>> =
            test::read_body_json(resp).await;

        let expected = vec![
            vec![
                ActionTrackWithActionName {
                    id: action_track_1.id,
                    action_id: None,
                    action_name: None,
                    started_at: action_track_1.started_at,
                    ended_at: action_track_1.ended_at,
                    duration: action_track_1.duration,
                },
                ActionTrackWithActionName {
                    id: action_track_0.id,
                    action_id: None,
                    action_name: None,
                    started_at: action_track_0.started_at,
                    ended_at: action_track_0.ended_at,
                    duration: action_track_0.duration,
                },
            ],
            vec![ActionTrackWithActionName {
                id: action_track_2.id,
                action_id: Some(action.id),
                action_name: Some(action.name),
                started_at: action_track_2.started_at,
                ended_at: action_track_2.ended_at,
                duration: action_track_2.duration,
            }],
        ];

        assert_eq!(returned_action_tracks.len(), expected.len());
        assert_eq!(returned_action_tracks[0], expected[0]);
        assert_eq!(returned_action_tracks[1], expected[1]);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::get().uri("/by_date").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
