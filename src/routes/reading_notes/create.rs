use ::types::ReadingNoteVisible;
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::reading_note_mutation::{NewReadingNote, ReadingNoteMutation};
use types::ReadingNoteCreateRequest;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating a reading note", skip(db, user))]
#[post("")]
pub async fn create_reading_note(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ReadingNoteCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ReadingNoteMutation::create(
                &db,
                NewReadingNote {
                    title: req.title.clone(),
                    page_number: req.page_number,
                    text: req.text.clone(),
                    date: req.date,
                    tag_ids: req.tag_ids.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(reading_note) => {
                    let res: ReadingNoteVisible = reading_note.into();
                    HttpResponse::Created().json(res)
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
