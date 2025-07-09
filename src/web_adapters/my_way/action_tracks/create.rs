use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

use crate::utils::{response_401, response_404, response_409, response_500};
use db_adapters::{
    action_adapter::ActionAdapter, action_track_adapter::ActionTrackAdapter,
    user_adapter::UserAdapter,
};
use entities::user as user_entity;
use use_cases::{
    my_way::action_tracks::{create::create_action_track, types::ActionTrackCreateRequest},
    UseCaseError,
};

#[tracing::instrument(name = "Creating an action track", skip(db, user))]
#[post("")]
pub async fn create_action_track_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_action_track(
                user.into_inner(),
                req.into_inner(),
                ActionTrackAdapter::init(&db),
                ActionAdapter::init(&db),
                UserAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    UseCaseError::Conflict(message) => response_409(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
