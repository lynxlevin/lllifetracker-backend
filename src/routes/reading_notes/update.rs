use ::types::{CustomDbErr, ReadingNoteVisible};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr, TransactionError};
use services::reading_note_mutation::{ReadingNoteMutation, UpdateReadingNote};
use types::ReadingNoteUpdateRequest;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    reading_note_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating a reading note", skip(db, user, req, path_param))]
#[put("/{reading_note_id}")]
pub async fn update_reading_note(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ReadingNoteUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let form = UpdateReadingNote {
                id: path_param.reading_note_id,
                title: req.title.clone(),
                page_number: req.page_number,
                text: req.text.clone(),
                date: req.date,
                tag_ids: req.tag_ids.clone(),
                user_id: user.id,
            };
            match ReadingNoteMutation::partial_update(&db, form).await {
                Ok(reading_note) => {
                    let res: ReadingNoteVisible = reading_note.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => match &e {
                    TransactionError::Transaction(DbErr::Custom(message)) => {
                        match message.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                response_404("Reading note with this id was not found")
                            }
                            _ => response_500(e),
                        }
                    }
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
