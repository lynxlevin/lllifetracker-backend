use crate::{
    entities::user as user_entity,
    services::mission_memo_mutation::MissionMemoMutation,
    types::{self, CustomDbErr, MissionMemoVisible, INTERNAL_SERVER_ERROR_MESSAGE},
};
use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    mission_memo_id: uuid::Uuid,
}

#[tracing::instrument(name = "Archiving a mission memo", skip(db, user, path_param))]
#[put("/{mission_memo_id}/archive")]
pub async fn archive_mission_memo(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match MissionMemoMutation::archive(&db, path_param.mission_memo_id, user.id).await {
                Ok(mission_memo) => HttpResponse::Ok().json(MissionMemoVisible {
                    id: mission_memo.id,
                    title: mission_memo.title,
                    text: mission_memo.text,
                    date: mission_memo.date,
                    archived: mission_memo.archived,
                    accomplished_at: mission_memo.accomplished_at,
                    created_at: mission_memo.created_at,
                    updated_at: mission_memo.updated_at,
                }),
                Err(e) => match e {
                    DbErr::Custom(e) => match e.parse::<CustomDbErr>().unwrap() {
                        CustomDbErr::NotFound => {
                            HttpResponse::NotFound().json(types::ErrorResponse {
                                error: "Mission Memo with this id was not found".to_string(),
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

    use crate::{entities::mission_memo, test_utils};

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
                .service(archive_mission_memo)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo =
            test_utils::seed::create_mission_memo(&db, "Mission Memo".to_string(), user.id).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", mission_memo.id))
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_mission_memo: MissionMemoVisible = test::read_body_json(res).await;
        assert_eq!(returned_mission_memo.title, mission_memo.title.clone());
        assert_eq!(returned_mission_memo.text, mission_memo.text.clone());
        assert_eq!(returned_mission_memo.date, mission_memo.date);
        assert_eq!(returned_mission_memo.archived, true);
        assert_eq!(
            returned_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(returned_mission_memo.created_at, mission_memo.created_at);
        assert!(returned_mission_memo.updated_at > mission_memo.updated_at);

        let updated_mission_memo = mission_memo::Entity::find_by_id(returned_mission_memo.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_mission_memo.title, mission_memo.title.clone());
        assert_eq!(updated_mission_memo.text, mission_memo.text.clone());
        assert_eq!(updated_mission_memo.date, mission_memo.date);
        assert_eq!(updated_mission_memo.archived, true);
        assert_eq!(
            updated_mission_memo.accomplished_at,
            mission_memo.accomplished_at
        );
        assert_eq!(updated_mission_memo.user_id, user.id);
        assert_eq!(updated_mission_memo.created_at, mission_memo.created_at);
        assert!(updated_mission_memo.updated_at > mission_memo.updated_at);

        Ok(())
    }

    #[actix_web::test]
    async fn not_found_if_invalid_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = test_utils::seed::create_active_user(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", uuid::Uuid::new_v4()))
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
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo = test_utils::seed::create_mission_memo(
            &db,
            "Mission Memo without tags".to_string(),
            user.id,
        )
        .await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}/archive", mission_memo.id))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
