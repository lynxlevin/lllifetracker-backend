use db_adapters::{
    diary_adapter::{
        DiaryAdapter, DiaryFilter, DiaryJoin, DiaryMutation, DiaryQuery, DiaryUpdateKey,
        UpdateDiaryParams,
    },
    CustomDbErr,
};
use entities::{diary, tag, user as user_entity};
use sea_orm::DbErr;
use uuid::Uuid;

use crate::{
    journal::diaries::types::{DiaryUpdateRequest, DiaryVisible},
    UseCaseError,
};

pub async fn update_diary<'a>(
    user: user_entity::Model,
    params: DiaryUpdateRequest,
    diary_id: Uuid,
    diary_adapter: DiaryAdapter<'a>,
) -> Result<DiaryVisible, UseCaseError> {
    let (diary, linked_tags) = diary_adapter
        .clone()
        .join_my_way_tags()
        .filter_eq_id(diary_id)
        .filter_eq_user(&user)
        .get_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Diary with this id was not found".to_string(),
        ))?;

    let diary = match diary_adapter
        .clone()
        .partial_update(
            diary,
            UpdateDiaryParams {
                text: params.text.clone(),
                date: params.date,
                update_keys: params.update_keys.clone(),
            },
        )
        .await
    {
        Ok(diary) => diary,
        Err(e) => match &e {
            _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
        },
    };

    if params.update_keys.contains(&DiaryUpdateKey::TagIds) {
        if let Err(e) =
            _update_tag_links(&diary, linked_tags, params.tag_ids.clone(), diary_adapter).await
        {
            // FIXME: diary creation should be canceled.
            match &e {
                DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                    CustomDbErr::NotFound => {
                        return Err(UseCaseError::NotFound(
                            "One or more of the tag_ids do not exist.".to_string(),
                        ))
                    }
                    _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
                },
                _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
            }
        };
    }
    Ok(DiaryVisible::from(diary))
}

async fn _update_tag_links(
    diary: &diary::Model,
    linked_tags: Vec<tag::Model>,
    tag_ids: Vec<Uuid>,
    diary_adapter: DiaryAdapter<'_>,
) -> Result<(), DbErr> {
    let linked_tag_ids = linked_tags.iter().map(|tag| tag.id).collect::<Vec<_>>();

    let tag_ids_to_link = tag_ids
        .clone()
        .into_iter()
        .filter(|id| !linked_tag_ids.contains(id));
    diary_adapter.link_tags(diary, tag_ids_to_link).await?;

    let tag_ids_to_delete = linked_tag_ids
        .into_iter()
        .filter(|linked_tag_id| !tag_ids.contains(linked_tag_id));
    diary_adapter.unlink_tags(diary, tag_ids_to_delete).await
}
