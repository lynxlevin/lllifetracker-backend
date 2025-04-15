use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::reading_note_mutation::ReadingNoteMutation;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    reading_note_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a reading note", skip(db, user, path_param))]
#[delete("/{reading_note_id}")]
pub async fn delete_reading_note(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ReadingNoteMutation::delete(&db, path_param.reading_note_id, user.id).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}
