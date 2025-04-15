use types::ReadingNoteCreateRequest;
use ::types::{self, ReadingNoteVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::reading_note_mutation::{NewReadingNote, ReadingNoteMutation};

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
