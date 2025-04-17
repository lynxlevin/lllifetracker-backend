use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::{tag, user as user_entity};
use sea_orm::{DbConn, DbErr};
use services::{tag_mutation::{TagMutation, UpdateTag}, tag_query::TagQuery};
use types::{CustomDbErr, TagUpdateRequest};

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
            match TagQuery::find_by_id_and_user_id(&db, path_param.tag_id, user.id).await {
                Ok(tag) => {
                    match _is_plain_tag(&tag) {
                        true => {
                            match TagMutation::update(
                                &db,
                                tag,
                                UpdateTag {
                                    name: req.name.clone(),
                                },
                            )
                            .await
                            {
                                Ok(tag) => HttpResponse::Ok().json(tag),
                                Err(e) => {
                                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                                    return HttpResponse::InternalServerError().json(types::ErrorResponse {
                                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                                    })
                                }
                            }
                        }
                        false => return HttpResponse::BadRequest().json(types::ErrorResponse {
                            error: "Tag to update must be a plain tag.".to_string(),
                        })
                    }
                }
                Err(e) => {
                    match &e {
                        DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                return HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "Tag with this id was not found".to_string(),
                                })
                            }
                            _ => {}
                        }
                        _ => {}
                    }
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    return HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}

fn _is_plain_tag(tag: &tag::Model) -> bool {
    return tag.name.is_some() && tag.ambition_id.is_none() && tag.desired_state_id.is_none() && tag.action_id.is_none()
}
