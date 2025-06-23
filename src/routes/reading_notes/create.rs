use ::types::ReadingNoteVisible;
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::{
    reading_note_adapter::{CreateReadingNoteParams, ReadingNoteAdapter, ReadingNoteMutation},
    CustomDbErr,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use types::ReadingNoteCreateRequest;

use crate::utils::{response_401, response_404, response_500};

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
            let reading_note = match ReadingNoteAdapter::init(&db)
                .create(CreateReadingNoteParams {
                    title: req.title.clone(),
                    page_number: req.page_number,
                    text: req.text.clone(),
                    date: req.date,
                    user_id: user.id,
                })
                .await
            {
                Ok(reading_note) => reading_note,
                Err(e) => return response_500(e),
            };
            match ReadingNoteAdapter::init(&db)
                .link_tags(&reading_note, req.tag_ids.clone())
                .await
            {
                Ok(_) => HttpResponse::Created().json(ReadingNoteVisible::from(reading_note)),
                // FIXME: reading_note creation should be canceled.
                Err(e) => match &e {
                    DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                        CustomDbErr::NotFound => {
                            response_404("One or more of the tag_ids do not exist.")
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
