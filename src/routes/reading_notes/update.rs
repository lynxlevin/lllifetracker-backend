use ::types::ReadingNoteVisible;
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::{
    reading_note_adapter::{
        ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteMutation, ReadingNoteQuery,
        UpdateReadingNoteParams,
    },
    CustomDbErr,
};
use entities::{reading_note, tag, user as user_entity};
use sea_orm::{DbConn, DbErr};
use types::ReadingNoteUpdateRequest;
use uuid::Uuid;

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
            let (reading_note, linked_tags) = match ReadingNoteAdapter::init(&db)
                .filter_eq_user(&user)
                .get_with_tags_by_id(path_param.reading_note_id)
                .await
            {
                Ok(res) => match res {
                    Some(res) => res,
                    None => return response_404("Reading note with this id was not found"),
                },
                Err(e) => return response_500(e),
            };

            let reading_note = match ReadingNoteAdapter::init(&db)
                .partial_update(
                    reading_note,
                    UpdateReadingNoteParams {
                        title: req.title.clone(),
                        page_number: req.page_number,
                        text: req.text.clone(),
                        date: req.date,
                    },
                )
                .await
            {
                Ok(reading_note) => reading_note,
                Err(e) => return response_500(e),
            };

            if let Some(tag_ids) = req.tag_ids.clone() {
                if let Err(e) = _update_tag_links(
                    &reading_note,
                    linked_tags,
                    tag_ids,
                    ReadingNoteAdapter::init(&db),
                )
                .await
                {
                    match &e {
                        DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                            CustomDbErr::NotFound => {
                                return response_404("One or more of the tag_ids do not exist.")
                            }
                            _ => return response_500(e),
                        },
                        // FIXME: reading_note creation should be canceled.
                        _ => return response_500(e),
                    }
                }
            }

            HttpResponse::Ok().json(ReadingNoteVisible::from(reading_note))
        }
        None => response_401(),
    }
}

async fn _update_tag_links(
    reading_note: &reading_note::Model,
    linked_tags: Vec<tag::Model>,
    tag_ids: Vec<Uuid>,
    reading_note_adapter: ReadingNoteAdapter<'_>,
) -> Result<(), DbErr> {
    let linked_tag_ids = linked_tags.iter().map(|tag| tag.id).collect::<Vec<_>>();

    let tag_ids_to_link = tag_ids
        .clone()
        .into_iter()
        .filter(|id| !linked_tag_ids.contains(id));
    reading_note_adapter
        .link_tags(&reading_note, tag_ids_to_link)
        .await?;

    let tag_ids_to_unlink = linked_tag_ids
        .into_iter()
        .filter(|id| !tag_ids.contains(id));
    reading_note_adapter
        .unlink_tags(&reading_note, tag_ids_to_unlink)
        .await?;

    Ok(())
}
