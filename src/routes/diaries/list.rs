use ::types::{DiaryVisibleWithTags, DiaryWithTagQueryResult, TagType, TagVisible};
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::diary_query::DiaryQuery;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing user's diaries.", skip(db, user))]
#[get("")]
pub async fn list_diaries(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DiaryQuery::find_all_with_tags_by_user_id(&db, user.id).await {
                Ok(diaries) => {
                    let mut res: Vec<DiaryVisibleWithTags> = vec![];
                    for diary in diaries {
                        if is_first_diary_to_process(&res, &diary) {
                            let mut res_diary = DiaryVisibleWithTags {
                                id: diary.id,
                                text: diary.text.clone(),
                                date: diary.date,
                                score: diary.score,
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
                    HttpResponse::Ok().json(res)
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}

fn is_first_diary_to_process(
    res: &Vec<DiaryVisibleWithTags>,
    diary: &DiaryWithTagQueryResult,
) -> bool {
    res.is_empty() || res.last().unwrap().id != diary.id
}

fn get_tag(diary: &DiaryWithTagQueryResult) -> Option<TagVisible> {
    if diary.tag_id.is_none() {
        return None;
    }
    let (name, tag_type) = if let Some(name) = diary.tag_name.clone() {
        (name, TagType::Plain)
    } else if let Some(name) = diary.tag_ambition_name.clone() {
        (name, TagType::Ambition)
    } else if let Some(name) = diary.tag_desired_state_name.clone() {
        (name, TagType::DesiredState)
    } else if let Some(name) = diary.tag_mindset_name.clone() {
        (name, TagType::Mindset)
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
