use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::{db::Db, errors::error_responses::response_400};
use domain_services::action_track_service::ActionTrackService;

use common::errors::error_responses::{response_401, response_404, response_500};
use db_adapters::{action_track_adapter::ActionTrackAdapter, user_adapter::UserAdapter};
use entities::user as user_entity;
use use_cases::my_way::action_tracks::{
    create::{create_action_track, CreateActionTracksError},
    types::ActionTrackCreateRequest,
};

#[post("")]
pub async fn create_action_track_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_action_track(
                user.into_inner(),
                req.into_inner(),
                ActionTrackAdapter::init(&db),
                UserAdapter::init(&db),
                ActionTrackService::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => match &e {
                    CreateActionTracksError::ValidationError(message) => response_400(message),
                    CreateActionTracksError::NotFound(message) => response_404(message),
                    CreateActionTracksError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
