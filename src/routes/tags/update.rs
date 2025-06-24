use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::tag_adapter::{
    TagAdapter, TagFilter, TagMutation, TagQuery, UpdatePlainTagParams,
};
use entities::{tag, user as user_entity};
use sea_orm::DbConn;
use types::{TagType, TagUpdateRequest, TagVisible};

use crate::utils::{response_400, response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    tag_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating a plain tag", skip(db, user))]
#[put("/plain/{tag_id}")]
pub async fn update_plain_tag(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<TagUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let tag = match TagAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.tag_id)
                .await
            {
                Ok(tag) => match tag {
                    Some(tag) => tag,
                    None => return response_404("Tag with this id was not found"),
                },
                Err(e) => return response_500(e),
            };
            if !_is_plain_tag(&tag) {
                return response_400("Tag to update must be a plain tag.");
            };

            match TagAdapter::init(&db)
                .update_plain(
                    tag,
                    UpdatePlainTagParams {
                        name: req.name.clone(),
                    },
                )
                .await
            {
                Ok(tag) => HttpResponse::Ok().json(TagVisible {
                    id: tag.id,
                    name: tag.name.unwrap(),
                    tag_type: TagType::Plain,
                    created_at: tag.created_at,
                }),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}

fn _is_plain_tag(tag: &tag::Model) -> bool {
    return tag.name.is_some()
        && tag.ambition_id.is_none()
        && tag.desired_state_id.is_none()
        && tag.action_id.is_none();
}
