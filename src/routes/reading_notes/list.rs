use ::types::{ReadingNoteVisibleWithTags, TagType, TagVisible};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::{
    reading_note_adapter::{
        ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteJoin, ReadingNoteOrder, ReadingNoteQuery,
        ReadingNoteWithTag,
    },
    Order::{Asc, Desc},
};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing user's reading notes.", skip(db, user))]
#[get("")]
pub async fn list_reading_notes(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ReadingNoteAdapter::init(&db)
                .filter_eq_user(&user)
                .join_my_way_tags()
                .order_by_date(Desc)
                .order_by_created_at(Desc)
                .order_by_ambition_created_at_nulls_last(Asc)
                .order_by_desired_state_created_at_nulls_last(Asc)
                .order_by_action_created_at_nulls_last(Asc)
                .order_by_tag_created_at_nulls_last(Asc)
                .get_all_with_tags()
                .await
            {
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
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}

fn get_tag(reading_note: &ReadingNoteWithTag) -> Option<TagVisible> {
    if reading_note.tag_id.is_none() {
        return None;
    }

    let (name, tag_type) = if let Some(name) = reading_note.tag_name.clone() {
        (name, TagType::Plain)
    } else if let Some(name) = reading_note.tag_ambition_name.clone() {
        (name, TagType::Ambition)
    } else if let Some(name) = reading_note.tag_desired_state_name.clone() {
        (name, TagType::DesiredState)
    } else if let Some(name) = reading_note.tag_action_name.clone() {
        (name, TagType::Action)
    } else {
        panic!("Tag without name should not exist.");
    };

    Some(TagVisible {
        id: reading_note.tag_id.unwrap(),
        name,
        tag_type,
        created_at: reading_note.tag_created_at.unwrap(),
    })
}
