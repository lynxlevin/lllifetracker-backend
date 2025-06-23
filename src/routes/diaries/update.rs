use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::{
    diary_adapter::{
        DiaryAdapter, DiaryFilter, DiaryMutation, DiaryQuery, DiaryUpdateKey, UpdateDiaryParams,
    },
    CustomDbErr,
};
use entities::{diary, tag, user as user_entity};
use sea_orm::{DbConn, DbErr};
use types::{DiaryUpdateRequest, DiaryVisible};
use uuid::Uuid;

use crate::utils::{response_400, response_401, response_404, response_409, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    diary_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating a diary", skip(db, user, req, path_param))]
#[put("/{diary_id}")]
pub async fn update_diary(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DiaryUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            if let Err(e) = _validate_request_body(&req) {
                return response_400(&e);
            };

            let (diary, linked_tags) = match DiaryAdapter::init(&db)
                .filter_eq_user(&user)
                .get_with_tags_by_id(path_param.diary_id)
                .await
            {
                Ok(res) => match res {
                    Some(res) => res,
                    None => return response_404("Diary with this id was not found"),
                },
                Err(e) => return response_500(e),
            };
            let diary = match DiaryAdapter::init(&db)
                .partial_update(
                    diary,
                    UpdateDiaryParams {
                        text: req.text.clone(),
                        date: req.date,
                        score: req.score,
                        update_keys: req.update_keys.clone(),
                    },
                )
                .await
            {
                Ok(diary) => diary,
                Err(e) => match &e {
                    DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                        CustomDbErr::Duplicate => {
                            return response_409(
                                "Another diary record for the same date already exists.",
                            )
                        }
                        _ => return response_500(e),
                    },
                    _ => return response_500(e),
                },
            };

            if req.update_keys.contains(&DiaryUpdateKey::TagIds) {
                if let Err(e) = _update_tag_links(
                    &diary,
                    linked_tags,
                    req.tag_ids.clone(),
                    DiaryAdapter::init(&db),
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
                        _ => return response_500(e),
                    }
                };
            }
            HttpResponse::Ok().json(DiaryVisible::from(diary))
        }
        None => response_401(),
    }
}

fn _validate_request_body(req: &DiaryUpdateRequest) -> Result<(), String> {
    if req.score.is_some_and(|score| score > 5 || score < 1) {
        return Err("score should be within 1 to 5.".to_string());
    }
    Ok(())
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
