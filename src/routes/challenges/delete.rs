use entities::user as user_entity;
use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use services::challenge_mutation::ChallengeMutation;
use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    challenge_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a challenge", skip(db, user, path_param))]
#[delete("/{challenge_id}")]
pub async fn delete_challenge(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ChallengeMutation::delete(&db, path_param.challenge_id, user.id).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json(types::ErrorResponse {
            error: "You are not logged in".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test, App, HttpMessage,
    };
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait};

    use entities::{challenge, challenges_tags};
    use test_utils::{self, *};

    use super::*;

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(delete_challenge)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_challenge_tag(&db, challenge.id, ambition_tag.id).await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", challenge.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let challenge_in_db = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?;
        assert!(challenge_in_db.is_none());

        let challenges_tags_in_db = challenges_tags::Entity::find()
            .filter(challenges_tags::Column::ChallengeId.eq(challenge.id))
            .filter(challenges_tags::Column::TagId.eq(ambition_tag.id))
            .one(&db)
            .await?;
        assert!(challenges_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let req = test::TestRequest::delete()
            .uri(&format!("/{}", challenge.id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
