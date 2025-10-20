use actix_web::{http, test, HttpMessage};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::journal::{
    diaries::types::DiaryVisibleWithTags, reading_notes::types::ReadingNoteVisibleWithTags,
    thinking_notes::types::ThinkingNoteVisibleWithTags, types::JournalVisibleWithTags,
};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn test_order() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let now = Utc::now();
    // NOTE: Active thinking_note comes first
    let thinking_note = factory::thinking_note(user.id)
        .updated_at((now - Duration::days(4)).into())
        .insert(&db)
        .await?;
    let diary = factory::diary(user.id).text(None).insert(&db).await?;
    // NOTE: resolved_thinking_note order is desc by resolved_at
    let resolved_thinking_note_0 = factory::thinking_note(user.id)
        .updated_at((now - Duration::days(3)).into())
        .resolved_at(Some((now - Duration::days(1)).into()))
        .insert(&db)
        .await?;
    let reading_note = factory::reading_note(user.id)
        .date((now - Duration::days(2)).date_naive())
        .title("reading_note_0".to_string())
        .insert(&db)
        .await?;
    let resolved_thinking_note_1 = factory::thinking_note(user.id)
        .updated_at((now - Duration::days(1)).into())
        .resolved_at(Some((now - Duration::days(3)).into()))
        .insert(&db)
        .await?;
    let _archived_thinking_note = factory::thinking_note(user.id)
        .archived_at(Some(Utc::now().into()))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get().uri("/api/journals").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<JournalVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        JournalVisibleWithTags {
            diary: None,
            reading_note: None,
            thinking_note: Some(ThinkingNoteVisibleWithTags {
                id: thinking_note.id,
                question: thinking_note.question,
                thought: thinking_note.thought,
                answer: thinking_note.answer,
                resolved_at: thinking_note.resolved_at,
                archived_at: thinking_note.archived_at,
                created_at: thinking_note.created_at,
                updated_at: thinking_note.updated_at,
                tags: vec![],
            }),
        },
        JournalVisibleWithTags {
            diary: Some(DiaryVisibleWithTags {
                id: diary.id,
                text: diary.text.clone(),
                date: diary.date,
                tags: vec![],
            }),
            reading_note: None,
            thinking_note: None,
        },
        JournalVisibleWithTags {
            diary: None,
            reading_note: None,
            thinking_note: Some(ThinkingNoteVisibleWithTags {
                id: resolved_thinking_note_0.id,
                question: resolved_thinking_note_0.question,
                thought: resolved_thinking_note_0.thought,
                answer: resolved_thinking_note_0.answer,
                resolved_at: resolved_thinking_note_0.resolved_at,
                archived_at: resolved_thinking_note_0.archived_at,
                created_at: resolved_thinking_note_0.created_at,
                updated_at: resolved_thinking_note_0.updated_at,
                tags: vec![],
            }),
        },
        JournalVisibleWithTags {
            diary: None,
            reading_note: Some(ReadingNoteVisibleWithTags {
                id: reading_note.id,
                title: reading_note.title.clone(),
                page_number: reading_note.page_number,
                text: reading_note.text.clone(),
                date: reading_note.date,
                created_at: reading_note.created_at,
                updated_at: reading_note.updated_at,
                tags: vec![],
            }),
            thinking_note: None,
        },
        JournalVisibleWithTags {
            diary: None,
            reading_note: None,
            thinking_note: Some(ThinkingNoteVisibleWithTags {
                id: resolved_thinking_note_1.id,
                question: resolved_thinking_note_1.question,
                thought: resolved_thinking_note_1.thought,
                answer: resolved_thinking_note_1.answer,
                resolved_at: resolved_thinking_note_1.resolved_at,
                archived_at: resolved_thinking_note_1.archived_at,
                created_at: resolved_thinking_note_1.created_at,
                updated_at: resolved_thinking_note_1.updated_at,
                tags: vec![],
            }),
        },
    ];

    assert_eq!(body.len(), expected.len());
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}

#[actix_web::test]
async fn test_tag_query() -> Result<(), DbErr> {
    todo!();
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri("/api/journals").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
