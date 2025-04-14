use ::types::{self, ReadingNoteVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::reading_note_mutation::{NewReadingNote, ReadingNoteMutation};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    title: String,
    page_number: i16,
    text: String,
    date: chrono::NaiveDate,
    tag_ids: Vec<uuid::Uuid>,
}

#[tracing::instrument(name = "Creating a reading note", skip(db, user))]
#[post("")]
pub async fn create_reading_note(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ReadingNoteMutation::create(
                &db,
                NewReadingNote {
                    title: req.title.clone(),
                    page_number: req.page_number,
                    text: req.text.clone(),
                    date: req.date,
                    tag_ids: req.tag_ids.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(reading_note) => {
                    let res: ReadingNoteVisible = reading_note.into();
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

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        dev::{Service, ServiceResponse},
        http, test,
        web::scope,
        App, HttpMessage,
    };
    use sea_orm::{
        entity::prelude::ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter,
        QueryFilter, QuerySelect,
    };

    use entities::{reading_note, reading_notes_tags};
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
                .service(scope("/").service(create_reading_note))
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let (_, tag_0) = factory::action(user.id)
            .name("action_0".to_string())
            .insert_with_tag(&db)
            .await?;
        let (_, tag_1) = factory::action(user.id)
            .name("action_1".to_string())
            .insert_with_tag(&db)
            .await?;

        let reading_note_title = "New reading note".to_string();
        let page_number = 12;
        let reading_note_text = "This is a new reading note for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                title: reading_note_title.clone(),
                page_number: page_number,
                text: reading_note_text.clone(),
                date: today,
                tag_ids: vec![tag_0.id, tag_1.id],
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::CREATED);

        let res: ReadingNoteVisible = test::read_body_json(res).await;
        assert_eq!(res.title, reading_note_title.clone());
        assert_eq!(res.page_number, page_number);
        assert_eq!(res.text, reading_note_text.clone());
        assert_eq!(res.date, today);

        let reading_note_in_db = reading_note::Entity::find_by_id(res.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(reading_note_in_db.user_id, user.id);
        assert_eq!(ReadingNoteVisible::from(reading_note_in_db), res);

        let linked_tag_ids: Vec<uuid::Uuid> = reading_notes_tags::Entity::find()
            .column_as(reading_notes_tags::Column::TagId, QueryAs::TagId)
            .filter(reading_notes_tags::Column::ReadingNoteId.eq(res.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 2);
        assert!(linked_tag_ids.contains(&tag_0.id));
        assert!(linked_tag_ids.contains(&tag_1.id));

        Ok(())
    }

    #[actix_web::test]
    async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(RequestBody {
                title: "New ReadingNote".to_string(),
                page_number: 1,
                text: "This is a new reading note for testing create method.".to_string(),
                date: chrono::Utc::now().date_naive(),
                tag_ids: vec![],
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
