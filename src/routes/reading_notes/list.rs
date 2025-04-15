use entities::user as user_entity;
use ::types::{
    self, ReadingNoteVisibleWithTags, ReadingNoteWithTagQueryResult, TagType, TagVisible,
    INTERNAL_SERVER_ERROR_MESSAGE,
};
use services::reading_note_query::ReadingNoteQuery;
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing user's reading notes.", skip(db, user))]
#[get("")]
pub async fn list_reading_notes(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ReadingNoteQuery::find_all_with_tags_by_user_id(&db, user.id).await {
                Ok(reading_notes) => {
                    let mut res: Vec<ReadingNoteVisibleWithTags> = vec![];
                    for reading_note in reading_notes {
                        if res.is_empty() || res.last().unwrap().id != reading_note.id {
                            let mut res_reading_note = ReadingNoteVisibleWithTags {
                                id: reading_note.id,
                                title: reading_note.title.clone(),
                                page_number: reading_note.page_number,
                                text: reading_note.text.clone(),
                                date: reading_note.date,
                                created_at: reading_note.created_at,
                                updated_at: reading_note.updated_at,
                                tags: vec![],
                            };
                            if let Some(tag) = get_tag(&reading_note) {
                                res_reading_note.push_tag(tag);
                            }
                            res.push(res_reading_note);
                        } else {
                            if let Some(tag) = get_tag(&reading_note) {
                                res.last_mut().unwrap().push_tag(tag);
                            }
                        }
                    }
                    HttpResponse::Ok().json(res)
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

fn get_tag(reading_note: &ReadingNoteWithTagQueryResult) -> Option<TagVisible> {
    if reading_note.tag_id.is_none() {
        return None;
    }

    if let Some(name) = reading_note.tag_ambition_name.clone() {
        Some(TagVisible {
            id: reading_note.tag_id.unwrap(),
            name,
            tag_type: TagType::Ambition,
            created_at: reading_note.tag_created_at.unwrap(),
        })
    } else if let Some(name) = reading_note.tag_desired_state_name.clone() {
        Some(TagVisible {
            id: reading_note.tag_id.unwrap(),
            name,
            tag_type: TagType::DesiredState,
            created_at: reading_note.tag_created_at.unwrap(),
        })
    } else if let Some(name) = reading_note.tag_action_name.clone() {
        Some(TagVisible {
            id: reading_note.tag_id.unwrap(),
            name,
            tag_type: TagType::Action,
            created_at: reading_note.tag_created_at.unwrap(),
        })
    } else {
        unimplemented!("Tag without link to Ambition/DesiredState/Action is not implemented yet.");
    }
}
