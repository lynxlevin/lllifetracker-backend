use actix_web::{http, test, HttpMessage};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter, QueryFilter, QuerySelect,
};
use use_cases::journal::reading_notes::types::{ReadingNoteUpdateRequest, ReadingNoteVisible};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{reading_note, reading_notes_tags};

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    let reading_note = factory::reading_note(user.id).insert(&db.db).await?;
    let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

    let form = ReadingNoteUpdateRequest {
        title: "reading note after update title".to_string(),
        page_number: 998,
        text: "reading note after update text".to_string(),
        date: chrono::NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
        tag_ids: vec![ambition_tag.id],
    };

    let req = test::TestRequest::put()
        .uri(&format!("/api/reading_notes/{}", reading_note.id))
        .set_json(&form)
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ReadingNoteVisible = test::read_body_json(res).await;
    assert_eq!(res.title, form.title.clone());
    assert_eq!(res.page_number, form.page_number);
    assert_eq!(res.text, form.text.clone());
    assert_eq!(res.date, form.date);
    assert_eq!(res.created_at, reading_note.created_at);
    assert!(res.updated_at > reading_note.updated_at);

    let reading_note_in_db = reading_note::Entity::find_by_id(res.id).one(&db.db).await?.unwrap();
    assert_eq!(reading_note_in_db.user_id, user.id);
    assert_eq!(ReadingNoteVisible::from(reading_note_in_db), res);

    let linked_tag_ids: Vec<uuid::Uuid> = reading_notes_tags::Entity::find()
        .column_as(reading_notes_tags::Column::TagId, QueryAs::TagId)
        .filter(reading_notes_tags::Column::ReadingNoteId.eq(res.id))
        .into_values::<_, QueryAs>()
        .all(&db.db)
        .await?;
    assert_eq!(linked_tag_ids.len(), 1);
    assert!(linked_tag_ids.contains(&ambition_tag.id));

    Ok(())
}

#[actix_web::test]
async fn not_found_if_invalid_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/reading_notes/{}", uuid::Uuid::now_v7()))
        .set_json(ReadingNoteUpdateRequest {
            title: String::default(),
            page_number: 0,
            text: String::default(),
            date: Utc::now().date_naive(),
            tag_ids: vec![],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    let reading_note = factory::reading_note(user.id).insert(&db.db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/reading_notes/{}", reading_note.id))
        .set_json(ReadingNoteUpdateRequest {
            title: reading_note.title,
            page_number: reading_note.page_number,
            text: reading_note.text,
            date: reading_note.date,
            tag_ids: vec![],
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_non_existent_tag_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    let reading_note = factory::reading_note(user.id).insert(&db.db).await?;

    let non_existent_tag_req = test::TestRequest::put()
        .uri(&format!("/api/reading_notes/{}", reading_note.id))
        .set_json(ReadingNoteUpdateRequest {
            title: reading_note.title,
            page_number: reading_note.page_number,
            text: reading_note.text,
            date: reading_note.date,
            tag_ids: vec![Uuid::now_v7()],
        })
        .to_request();
    non_existent_tag_req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, non_existent_tag_req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}
