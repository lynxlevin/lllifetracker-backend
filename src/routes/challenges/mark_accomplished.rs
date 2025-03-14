use entities::user as user_entity;
use ::types::{self, CustomDbErr, ChallengeVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use services::challenge_mutation::ChallengeMutation;
use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    challenge_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Marking a challenge as accomplished",
    skip(db, user, path_param)
)]
#[put("/{challenge_id}/accomplish")]
pub async fn mark_accomplished_challenge(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ChallengeMutation::mark_accomplished(&db, path_param.challenge_id, user.id)
                .await
            {
                Ok(challenge) => {
                    let res: ChallengeVisible = challenge.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => match e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "Challenge with this id was not found".to_string(),
                            })
                        }
                    },
                    e => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                        HttpResponse::InternalServerError().json(types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                },
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

    use entities::challenge;
    use test_utils::{self, *};

    use super::*;

    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    enum QueryAs {
        TagId,
    }

    async fn init_app(
        db: DbConn,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .service(mark_accomplished_challenge)
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

        let req = test::TestRequest::put()
            .uri(&format!("/{}/accomplish", challenge.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_challenge: ChallengeVisible = test::read_body_json(res).await;
        assert_eq!(returned_challenge.title, challenge.title.clone());
        assert_eq!(returned_challenge.text, challenge.text.clone());
        assert_eq!(returned_challenge.date, challenge.date);
        assert_eq!(returned_challenge.archived, challenge.archived);
        assert!(returned_challenge.accomplished_at.is_some());
        assert_eq!(returned_challenge.created_at, challenge.created_at);
        assert!(returned_challenge.updated_at > challenge.updated_at);

        let updated_challenge = challenge::Entity::find_by_id(returned_challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_challenge.title, challenge.title.clone());
        assert_eq!(updated_challenge.text, challenge.text.clone());
        assert_eq!(updated_challenge.date, challenge.date);
        assert_eq!(updated_challenge.archived, challenge.archived);
        assert!(updated_challenge.accomplished_at.is_some());
        assert_eq!(updated_challenge.user_id, user.id);
        assert_eq!(updated_challenge.created_at, challenge.created_at);
        assert!(updated_challenge.updated_at > challenge.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn not_found_if_invalid_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/accomplish", uuid::Uuid::new_v4()))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/accomplish", challenge.id))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
