use db_adapters::{
    diary_adapter::{DiaryAdapter, DiaryFilter, DiaryJoin, DiaryOrder, DiaryQuery, DiaryWithTag},
    Order::{Asc, Desc},
};
use entities::user as user_entity;

use crate::{
    journal::diaries::types::DiaryVisibleWithTags,
    tags::types::{TagType, TagVisible},
    UseCaseError,
};

pub async fn list_diaries<'a>(
    user: user_entity::Model,
    diary_adapter: DiaryAdapter<'a>,
) -> Result<Vec<DiaryVisibleWithTags>, UseCaseError> {
    let diaries = diary_adapter
        .join_tags()
        .join_my_way_via_tags()
        .filter_eq_user(&user)
        .order_by_date(Desc)
        .order_by_ambition_created_at_nulls_last(Asc)
        .order_by_desired_state_created_at_nulls_last(Asc)
        .order_by_action_created_at_nulls_last(Asc)
        .order_by_tag_created_at_nulls_last(Asc)
        .get_all_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<DiaryVisibleWithTags> = vec![];
    for diary in diaries {
        if is_first_diary_to_process(&res, &diary) {
            let mut res_diary = DiaryVisibleWithTags {
                id: diary.id,
                text: diary.text.clone(),
                date: diary.date,
                tags: vec![],
            };
            if let Some(tag) = get_tag(&diary) {
                res_diary.push_tag(tag);
            }
            res.push(res_diary);
        } else {
            if let Some(tag) = get_tag(&diary) {
                res.last_mut().unwrap().push_tag(tag);
            }
        }
    }
    Ok(res)
}

fn is_first_diary_to_process(res: &Vec<DiaryVisibleWithTags>, diary: &DiaryWithTag) -> bool {
    res.is_empty() || res.last().unwrap().id != diary.id
}

fn get_tag(diary: &DiaryWithTag) -> Option<TagVisible> {
    if diary.tag_id.is_none() {
        return None;
    }
    let (name, tag_type) = if let Some(name) = diary.tag_name.clone() {
        (name, TagType::Plain)
    } else if let Some(name) = diary.tag_ambition_name.clone() {
        (name, TagType::Ambition)
    } else if let Some(name) = diary.tag_desired_state_name.clone() {
        (name, TagType::DesiredState)
    } else if let Some(name) = diary.tag_action_name.clone() {
        (name, TagType::Action)
    } else {
        panic!("Tag without name should not exist.");
    };

    Some(TagVisible {
        id: diary.tag_id.unwrap(),
        name,
        tag_type,
        created_at: diary.tag_created_at.unwrap(),
    })
}
