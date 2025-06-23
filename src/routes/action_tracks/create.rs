use ::types::{self, ActionTrackVisible};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use chrono::SubsecRound;
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionQuery},
    action_track_adapter::{ActionTrackAdapter, ActionTrackMutation, CreateActionTrackParams},
    CustomDbErr,
};
use entities::{sea_orm_active_enums::ActionTrackType, user as user_entity};
use sea_orm::{DbConn, DbErr};
use types::ActionTrackCreateRequest;

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
            match ActionAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(req.action_id)
                .await
            {
                Ok(action) => match action {
                    Some(action) => {
                        let result = match action.track_type {
                            ActionTrackType::TimeSpan => {
                                ActionTrackAdapter::init(&db)
                                    .create(CreateActionTrackParams {
                                        started_at: req.started_at.trunc_subsecs(0),
                                        ended_at: None,
                                        duration: None,
                                        action_id: req.action_id,
                                        user_id: user.id,
                                    })
                                    .await
                            }
                            ActionTrackType::Count => {
                                ActionTrackAdapter::init(&db)
                                    .create(CreateActionTrackParams {
                                        started_at: req.started_at.trunc_subsecs(0),
                                        ended_at: Some(req.started_at.trunc_subsecs(0)),
                                        duration: Some(0),
                                        action_id: req.action_id,
                                        user_id: user.id,
                                    })
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
                                    match CustomDbErr::from(message) {
                                                CustomDbErr::Duplicate => response_409("A track for the same action which starts at the same time exists."),
                                                _ => response_500(e),
                                            }
                                }
                                _ => response_500(e),
                            },
                        }
                    }
                    None => response_404("An action with that id does not exist."),
                },
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
