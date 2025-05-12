use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::mindset_mutation::{MindsetMutation, NewMindset};
use types::{MindsetCreateRequest, MindsetVisible};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating an mindset", skip(db, user))]
#[post("")]
pub async fn create_mindset(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<MindsetCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MindsetMutation::create_with_tag(
                &db,
                NewMindset {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(mindset) => {
                    let res: MindsetVisible = mindset.into();
                    HttpResponse::Created().json(res)
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
