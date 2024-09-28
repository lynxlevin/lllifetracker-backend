use crate::{
    entities::user as user_entity,
    services::action::Query as ActionQuery,
    startup::AppState,
    types::{self, ActionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an action", skip(data, user))]
#[get("/{action_id}")]
pub async fn get_action(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionQuery::find_by_id_and_user_id(&data.conn, path_param.action_id, user.id)
                .await
            {
                Ok(action) => match action {
                    Some(action) => HttpResponse::Ok().json(ActionVisible {
                        id: action.id,
                        name: action.name,
                        created_at: action.created_at,
                        updated_at: action.updated_at,
                    }),
                    None => HttpResponse::NotFound().json(types::ErrorResponse {
                        error: "Action with this id was not found".to_string(),
                    }),
                },
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
