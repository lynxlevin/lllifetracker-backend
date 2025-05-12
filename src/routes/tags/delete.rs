use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::{tag, user as user_entity};
use sea_orm::{DbConn, DbErr};
use services::{tag_mutation::TagMutation, tag_query::TagQuery};
use types::CustomDbErr;

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
            match TagQuery::find_by_id_and_user_id(&db, path_param.tag_id, user.id).await {
                Ok(tag) => match _is_plain_tag(&tag) {
                    true => match TagMutation::delete(&db, tag).await {
                        Ok(_) => HttpResponse::NoContent().finish(),
                        Err(e) => response_500(e),
                    },
                    false => response_400("Tag to delete must be a plain tag."),
                },
                Err(e) => match &e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => return HttpResponse::NoContent().finish(),
                        _ => response_500(e),
                    },
                    _ => response_500(e),
                },
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
