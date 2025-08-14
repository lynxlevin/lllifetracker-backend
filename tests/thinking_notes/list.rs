use actix_web::{http, test, HttpMessage};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::{
    journal::thinking_notes::types::ThinkingNoteVisibleWithTags,
    tags::types::{TagType, TagVisible},
};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path_active_only() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let thinking_note_0 = factory::thinking_note(user.id)
        .question(Some("thinking_note_0".to_string()))
        .insert(&db)
        .await?;
    let thinking_note_1 = factory::thinking_note(user.id)
        .question(Some("thinking_note_1".to_string()))
        .insert(&db)
        .await?;
    let _resolved_thinking_note = factory::thinking_note(user.id)
        .resolved_at(Some(Utc::now().into()))
        .insert(&db)
        .await?;
    let _archived_thinking_note = factory::thinking_note(user.id)
        .archived_at(Some(Utc::now().into()))
        .insert(&db)
        .await?;
    let plain_tag = factory::tag(user.id).insert(&db).await?;
    let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
    let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    let (desired_state, desired_state_tag) =
        factory::desired_state(user.id).insert_with_tag(&db).await?;
    factory::link_thinking_note_tag(&db, thinking_note_0.id, plain_tag.id).await?;
    factory::link_thinking_note_tag(&db, thinking_note_0.id, ambition_tag.id).await?;
    factory::link_thinking_note_tag(&db, thinking_note_1.id, desired_state_tag.id).await?;
    factory::link_thinking_note_tag(&db, thinking_note_1.id, action_tag.id).await?;

    let req = test::TestRequest::get()
        .uri("/api/thinking_notes")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<ThinkingNoteVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        ThinkingNoteVisibleWithTags {
            id: thinking_note_1.id,
            question: thinking_note_1.question,
            thought: thinking_note_1.thought,
            answer: thinking_note_1.answer,
            resolved_at: thinking_note_1.resolved_at,
            archived_at: thinking_note_1.archived_at,
            created_at: thinking_note_1.created_at,
            updated_at: thinking_note_1.updated_at,
            tags: vec![
                TagVisible {
                    id: desired_state_tag.id,
                    name: desired_state.name,
                    tag_type: TagType::DesiredState,
                    created_at: desired_state_tag.created_at,
                },
                TagVisible {
                    id: action_tag.id,
                    name: action.name,
                    tag_type: TagType::Action,
                    created_at: action_tag.created_at,
                },
            ],
        },
        ThinkingNoteVisibleWithTags {
            id: thinking_note_0.id,
            question: thinking_note_0.question,
            thought: thinking_note_0.thought,
            answer: thinking_note_0.answer,
            resolved_at: thinking_note_0.resolved_at,
            archived_at: thinking_note_0.archived_at,
            created_at: thinking_note_0.created_at,
            updated_at: thinking_note_0.updated_at,
            tags: vec![
                TagVisible {
                    id: ambition_tag.id,
                    name: ambition.name,
                    tag_type: TagType::Ambition,
                    created_at: ambition_tag.created_at,
                },
                TagVisible {
                    id: plain_tag.id,
                    name: plain_tag.name.unwrap(),
                    tag_type: TagType::Plain,
                    created_at: plain_tag.created_at,
                },
            ],
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
async fn happy_path_resolved_only() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _active_thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    let resolved_thinking_note = factory::thinking_note(user.id)
        .resolved_at(Some(Utc::now().into()))
        .insert(&db)
        .await?;
    let _archived_thinking_note = factory::thinking_note(user.id)
        .archived_at(Some(Utc::now().into()))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/thinking_notes?resolved=true")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<ThinkingNoteVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![ThinkingNoteVisibleWithTags {
        id: resolved_thinking_note.id,
        question: resolved_thinking_note.question,
        thought: resolved_thinking_note.thought,
        answer: resolved_thinking_note.answer,
        resolved_at: resolved_thinking_note.resolved_at,
        archived_at: resolved_thinking_note.archived_at,
        created_at: resolved_thinking_note.created_at,
        updated_at: resolved_thinking_note.updated_at,
        tags: vec![],
    }];

    assert_eq!(body.len(), expected.len());
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}

#[actix_web::test]
async fn happy_path_archived_only() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _active_thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    let _resolved_thinking_note = factory::thinking_note(user.id)
        .resolved_at(Some(Utc::now().into()))
        .insert(&db)
        .await?;
    let archived_thinking_note = factory::thinking_note(user.id)
        .archived_at(Some(Utc::now().into()))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/thinking_notes?archived=true")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<ThinkingNoteVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![ThinkingNoteVisibleWithTags {
        id: archived_thinking_note.id,
        question: archived_thinking_note.question,
        thought: archived_thinking_note.thought,
        answer: archived_thinking_note.answer,
        resolved_at: archived_thinking_note.resolved_at,
        archived_at: archived_thinking_note.archived_at,
        created_at: archived_thinking_note.created_at,
        updated_at: archived_thinking_note.updated_at,
        tags: vec![],
    }];

    assert_eq!(body.len(), expected.len());
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/thinking_notes")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
