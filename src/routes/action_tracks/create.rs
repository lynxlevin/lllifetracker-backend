use ::types::{self, ActionTrackVisible};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::{sea_orm_active_enums::ActionTrackType, user as user_entity};
use sea_orm::{DbConn, DbErr};
use services::{
    action_query::ActionQuery,
    action_track_mutation::{ActionTrackMutation, NewActionTrack},
};
use types::{ActionTrackCreateRequest, CustomDbErr};

use crate::utils::{response_401, response_404, response_409, response_500};

#[tracing::instrument(name = "Creating an action track", skip(db, user))]
#[post("")]
pub async fn create_action_track(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionQuery::find_by_id_and_user_id(&db, req.action_id, user.id).await {
                Ok(action) => {
                    let result = match action.track_type {
                        ActionTrackType::TimeSpan => {
                            ActionTrackMutation::create(
                                &db,
                                NewActionTrack {
                                    started_at: req.started_at,
                                    ended_at: None,
                                    action_id: req.action_id,
                                    user_id: user.id,
                                },
                            )
                            .await
                        }
                        ActionTrackType::Count => {
                            ActionTrackMutation::create(
                                &db,
                                NewActionTrack {
                                    started_at: req.started_at,
                                    ended_at: Some(req.started_at),
                                    action_id: req.action_id,
                                    user_id: user.id,
                                },
                            )
                            .await
                        }
                    };
                    match result {
                        Ok(action_track) => {
                            let res: ActionTrackVisible = action_track.into();
                            HttpResponse::Created().json(res)
                        }
                        Err(e) => match &e {
                            DbErr::Custom(message) => {
                                match message.parse::<CustomDbErr>().unwrap() {
                                        CustomDbErr::Duplicate => response_409("A track for the same action which starts at the same time exists."),
                                        _ => response_500(e),
                                    }
                            }
                            _ => response_500(e),
                        },
                    }
                }
                Err(e) => match &e {
                    DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            return response_404("An action with that id does not exist.")
                        }
                        _ => response_500(e),
                    },
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
