use entities::user as user_entity;
use ::types::{self, ReadingNoteVisible, CustomDbErr, INTERNAL_SERVER_ERROR_MESSAGE};
use services::reading_note_mutation::{ReadingNoteMutation, UpdateReadingNote};
use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::{DbConn, DbErr, TransactionError};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    reading_note_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    title: Option<String>,
    page_number: Option<i16>,
    text: Option<String>,
    date: Option<chrono::NaiveDate>,
    tag_ids: Option<Vec<uuid::Uuid>>,
}

#[tracing::instrument(name = "Updating a reading note", skip(db, user, req, path_param))]
#[put("/{reading_note_id}")]
pub async fn update_reading_note(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<RequestBody>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let form = UpdateReadingNote {
                id: path_param.reading_note_id,
                title: req.title.clone(),
                page_number: req.page_number,
                text: req.text.clone(),
                date: req.date,
                tag_ids: req.tag_ids.clone(),
                user_id: user.id,
            };
            match ReadingNoteMutation::partial_update(&db, form).await {
                Ok(reading_note) => {
                    let res: ReadingNoteVisible = reading_note.into();
                    HttpResponse::Ok().json(res)
                }
                Err(e) => match e {
                    TransactionError::Transaction(DbErr::Custom(message)) => {
                        match message.parse::<CustomDbErr>().unwrap() {
                            CustomDbErr::NotFound => {
                                HttpResponse::NotFound().json(types::ErrorResponse {
                                    error: "reading note with this id was not found".to_string(),
                                })
                            }
                        }
                    }
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
    use sea_orm::{entity::prelude::*, DbErr, EntityTrait, QuerySelect};

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
                .service(update_reading_note)
                .app_data(Data::new(db)),
        )
        .await
    }

    #[actix_web::test]
    async fn happy_path() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;
        let reading_note = factory::reading_note(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

        let form = RequestBody {
            title: Some("reading note after update title".to_string()),
            page_number: Some(998),
            text: Some("reading note after update text".to_string()),
            date: Some(chrono::NaiveDate::from_ymd_opt(2024, 11, 3).unwrap()),
            tag_ids: Some(vec![ambition_tag.id]),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/{}", reading_note.id))
            .set_json(&form)
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);

        let returned_reading_note: ReadingNoteVisible = test::read_body_json(res).await;
        assert_eq!(returned_reading_note.title, form.title.clone().unwrap());
        assert_eq!(returned_reading_note.page_number, form.page_number.unwrap());
        assert_eq!(returned_reading_note.text, form.text.clone().unwrap());
        assert_eq!(returned_reading_note.date, form.date.unwrap());
        assert_eq!(returned_reading_note.created_at, reading_note.created_at);
        assert!(returned_reading_note.updated_at > reading_note.updated_at);

        let updated_reading_note = reading_note::Entity::find_by_id(returned_reading_note.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_reading_note.title, form.title.unwrap());
        assert_eq!(updated_reading_note.page_number, form.page_number.unwrap());
        assert_eq!(updated_reading_note.text, form.text.unwrap());
        assert_eq!(updated_reading_note.date, form.date.unwrap());
        assert_eq!(updated_reading_note.user_id, user.id);
        assert_eq!(
            updated_reading_note.created_at,
            returned_reading_note.created_at
        );
        assert_eq!(
            updated_reading_note.updated_at,
            returned_reading_note.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = reading_notes_tags::Entity::find()
            .column_as(reading_notes_tags::Column::TagId, QueryAs::TagId)
            .filter(reading_notes_tags::Column::ReadingNoteId.eq(returned_reading_note.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 1);
        assert!(linked_tag_ids.contains(&ambition_tag.id));

        Ok(())
    }

    #[actix_web::test]
    async fn not_found_if_invalid_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let app = init_app(db.clone()).await;
        let user = factory::user().insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", uuid::Uuid::new_v4()))
            .set_json(RequestBody {
                title: None,
                page_number: None,
                text: None,
                date: None,
                tag_ids: None,
            })
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
        let reading_note = factory::reading_note(user.id).insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/{}", reading_note.id))
            .set_json(RequestBody {
                title: None,
                page_number: None,
                text: None,
                date: None,
                tag_ids: None,
            })
            .to_request();

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
