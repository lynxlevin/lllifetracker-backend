use crate::{
    entities::user as user_entity,
    services::ambition::Query as AmbitionQuery,
    startup::AppState,
    types::{self, AmbitionVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    ambition_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an ambition", skip(data, user))]
#[get("/{ambition_id}")]
pub async fn get_ambition(
    data: Data<AppState>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionQuery::find_by_id_and_user_id(&data.conn, path_param.ambition_id, user.id)
                .await
            {
                Ok(ambition) => match ambition {
                    Some(ambition) => HttpResponse::Ok().json(AmbitionVisible {
                        id: ambition.id,
                        name: ambition.name,
                        description: ambition.description,
                        created_at: ambition.created_at,
                        updated_at: ambition.updated_at,
                    }),
                    None => HttpResponse::NotFound().json(types::ErrorResponse {
                        error: "Ambition with this id was not found".to_string(),
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
