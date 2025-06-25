use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::{
    journal::reading_notes::types::ReadingNoteVisibleWithTags,
    tags::types::{TagType, TagVisible},
};

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let reading_note_0 = factory::reading_note(user.id)
        .title("reading_note_0".to_string())
        .insert(&db)
        .await?;
    let reading_note_1 = factory::reading_note(user.id)
        .title("reading_note_1".to_string())
        .insert(&db)
        .await?;
    let plain_tag = factory::tag(user.id).insert(&db).await?;
    let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
    let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    let (desired_state, desired_state_tag) =
        factory::desired_state(user.id).insert_with_tag(&db).await?;
    factory::link_reading_note_tag(&db, reading_note_0.id, plain_tag.id).await?;
    factory::link_reading_note_tag(&db, reading_note_0.id, ambition_tag.id).await?;
    factory::link_reading_note_tag(&db, reading_note_1.id, desired_state_tag.id).await?;
    factory::link_reading_note_tag(&db, reading_note_1.id, action_tag.id).await?;

    let req = test::TestRequest::get()
        .uri("/api/reading_notes")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<ReadingNoteVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        ReadingNoteVisibleWithTags {
            id: reading_note_1.id,
            title: reading_note_1.title.clone(),
            page_number: reading_note_1.page_number,
            text: reading_note_1.text.clone(),
            date: reading_note_1.date,
            created_at: reading_note_1.created_at,
            updated_at: reading_note_1.updated_at,
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
        ReadingNoteVisibleWithTags {
            id: reading_note_0.id,
            title: reading_note_0.title.clone(),
            page_number: reading_note_0.page_number,
            text: reading_note_0.text.clone(),
            date: reading_note_0.date,
            created_at: reading_note_0.created_at,
            updated_at: reading_note_0.updated_at,
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
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/reading_notes")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
