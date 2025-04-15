use types::ActionCreateRequest;
use ::types::{self, ActionVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::action_mutation::{ActionMutation, NewAction};

#[tracing::instrument(name = "Creating an action", skip(db, user))]
#[post("")]
pub async fn create_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionMutation::create_with_tag(
                &db,
                NewAction {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    track_type: req.track_type.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(action) => {
                    let res: ActionVisible = action.into();
                    HttpResponse::Created().json(res)
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
