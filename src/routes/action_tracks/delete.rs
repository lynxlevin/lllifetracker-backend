use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::{
    action_track_mutation::ActionTrackMutation,
    action_track_query::{ActionTrackQuery, ActionTrackQueryFilter},
};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_track_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an action track", skip(db, user))]
#[delete("/{action_track_id}")]
pub async fn delete_action_track(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let action_track = match ActionTrackQuery::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.action_track_id)
                .await
            {
                Ok(action_track) => match action_track {
                    Some(action_track) => action_track,
                    None => return HttpResponse::NoContent().finish(),
                },
                Err(e) => return response_500(e),
            };
            match ActionTrackMutation::init(&db).delete(action_track).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
