use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::tag_adapter::{TagAdapter, TagFilter, TagMutation, TagQuery};
use entities::{tag, user as user_entity};
use sea_orm::DbConn;

use crate::utils::{response_400, response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    tag_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a plain tag", skip(db, user))]
#[delete("/plain/{tag_id}")]
pub async fn delete_plain_tag(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
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
                    None => return HttpResponse::NoContent().finish(),
                },
                Err(e) => return response_500(e),
            };
            if !_is_plain_tag(&tag) {
                return response_400("Tag to delete must be a plain tag.");
            };
            match TagAdapter::init(&db).delete(tag).await {
                Ok(_) => HttpResponse::NoContent().finish(),
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
