use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::mindset_query::MindsetQuery;
use types::{CustomDbErr, MindsetVisible};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    mindset_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an mindset", skip(db, user))]
#[get("/{mindset_id}")]
pub async fn get_mindset(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MindsetQuery::find_by_id_and_user_id(&db, path_param.mindset_id, user.id).await {
                Ok(mindset) => {
                    let res: MindsetVisible = mindset.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => match &e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => response_404("Mindset with this id was not found"),
                        _ => response_500(e),
                    },
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
