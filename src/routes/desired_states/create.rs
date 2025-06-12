use ::types::DesiredStateVisible;
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::{
    desired_state_category_query::DesiredStateCategoryQuery,
    desired_state_mutation::{DesiredStateMutation, NewDesiredState},
};
use types::{CustomDbErr, DesiredStateCreateRequest};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating an desired_state", skip(db, user))]
#[post("")]
pub async fn create_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let category_id = match req.category_id {
                Some(category_id) => match DesiredStateCategoryQuery::find_by_id_and_user_id(
                    &db,
                    category_id,
                    user.id,
                )
                .await
                {
                    Ok(_) => Some(category_id),
                    Err(e) => match e {
                        DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => None,
                            _ => return response_500(e),
                        },
                        _ => return response_500(e),
                    },
                },
                None => None,
            };
            match DesiredStateMutation::create_with_tag(
                &db,
                NewDesiredState {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    category_id: category_id,
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(desired_state) => {
                    let res: DesiredStateVisible = desired_state.into();
                    HttpResponse::Created().json(res)
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
