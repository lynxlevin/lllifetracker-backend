use db_adapters::{
    diary_adapter::{DiaryAdapter, DiaryFilter, DiaryJoin, DiaryOrder, DiaryQuery, DiaryWithTag},
    Order::{Asc, Desc},
};
use entities::user as user_entity;

use crate::{journal::diaries::types::DiaryVisibleWithTags, tags::types::TagVisible, UseCaseError};

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
        if first_to_process(&res, &diary) {
            let tags = match diary.tag_id {
                Some(_) => vec![Into::<TagVisible>::into(&diary)],
                None => vec![],
            };
            let res_diary = DiaryVisibleWithTags {
                id: diary.id,
                text: diary.text,
                date: diary.date,
                tags,
            };
            res.push(res_diary);
        } else {
            if let Some(_) = diary.tag_id {
                res.last_mut()
                    .unwrap()
                    .push_tag(Into::<TagVisible>::into(&diary));
            }
        }
    }
    Ok(res)
}

fn first_to_process(res: &Vec<DiaryVisibleWithTags>, diary: &DiaryWithTag) -> bool {
    res.is_empty() || res.last().unwrap().id != diary.id
}
