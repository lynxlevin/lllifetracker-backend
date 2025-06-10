use ::types::{self, ActionVisible, CustomDbErr};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::{DbConn, DbErr};
use services::{action_mutation::ActionMutation, action_query::ActionQuery};
use types::ActionTrackTypeConversionRequest;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Converting action type", skip(db, user, req, path_param))]
#[put("/{action_id}/track_type")]
pub async fn convert_action_track_type(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackTypeConversionRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionQuery::find_by_id_and_user_id(&db, path_param.action_id, user.id).await {
                Ok(action) => match action.track_type == req.track_type {
                    true => HttpResponse::Ok().json(ActionVisible::from(action)),
                    false => {
                        match ActionMutation::convert_track_type(
                            &db,
                            action,
                            req.track_type.clone(),
                        )
                        .await
                        {
                            Ok(action) => HttpResponse::Ok().json(ActionVisible::from(action)),
                            Err(e) => response_500(e),
                        }
                    }
                },
                Err(e) => match &e {
                    DbErr::Custom(message) => match message.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => response_404("Action with this id was not found"),
                        _ => response_500(e),
                    },
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
