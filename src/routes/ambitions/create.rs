use types::AmbitionCreateRequest;
use ::types::{self, AmbitionVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::ambition_mutation::{AmbitionMutation, NewAmbition};

#[tracing::instrument(name = "Creating an ambition", skip(db, user))]
#[post("")]
pub async fn create_ambition(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<AmbitionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionMutation::create_with_tag(
                &db,
                NewAmbition {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(ambition) => {
                    let res: AmbitionVisible = ambition.into();
                    HttpResponse::Created().json(res)
                }
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}
