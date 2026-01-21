use actix_web::{http, test, HttpMessage};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, DbErr};

use crate::utils::{init_app, Connections};
use common::factory::{self, *};
use use_cases::{
    journal::{
        diaries::types::DiaryVisibleWithTags, reading_notes::types::ReadingNoteVisibleWithTags,
        thinking_notes::types::ThinkingNoteVisibleWithTags, types::JournalVisibleWithTags,
    },
    tags::types::TagVisible,
};

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
        .updated_at((now - Duration::days(1)).into())
        .resolved_at(Some((now - Duration::days(1)).into()))
        .insert(&db)
        .await?;
    let reading_note = factory::reading_note(user.id)
        .date((now - Duration::days(2)).date_naive())
        .title("reading_note_0".to_string())
        .insert(&db)
        .await?;
    let resolved_thinking_note_1 = factory::thinking_note(user.id)
        .updated_at((now - Duration::days(3)).into())
        .resolved_at(Some((now - Duration::days(3)).into()))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get().uri("/api/journals").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<JournalVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((thinking_note, vec![]))),
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((diary, vec![]))),
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((
            resolved_thinking_note_0,
            vec![],
        ))),
        JournalVisibleWithTags::from(ReadingNoteVisibleWithTags::from((reading_note, vec![]))),
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((
            resolved_thinking_note_1,
            vec![],
        ))),
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
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let plain_tag_0 = factory::tag(user.id).insert(&db).await?;
    let plain_tag_1 = factory::tag(user.id).insert(&db).await?;
    let plain_tag_2 = factory::tag(user.id).insert(&db).await?;

    let tagged_thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    let tagged_diary = factory::diary(user.id).insert(&db).await?;
    let tagged_reading_note = factory::reading_note(user.id).insert(&db).await?;
    factory::link_thinking_note_tag(&db, tagged_thinking_note.id, plain_tag_0.id).await?;
    factory::link_thinking_note_tag(&db, tagged_thinking_note.id, plain_tag_2.id).await?;
    factory::link_diary_tag(&db, tagged_diary.id, plain_tag_0.id).await?;
    factory::link_reading_note_tag(&db, tagged_reading_note.id, plain_tag_1.id).await?;

    let _thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    let _diary = factory::diary(user.id).insert(&db).await?;
    let _reading_note = factory::reading_note(user.id).insert(&db).await?;
    let different_tagged_thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    factory::link_thinking_note_tag(&db, different_tagged_thinking_note.id, plain_tag_2.id).await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/journals?tag_id_or={},{}",
            plain_tag_0.id, plain_tag_1.id
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<JournalVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((
            tagged_thinking_note,
            vec![
                TagVisible {
                    id: plain_tag_0.id,
                    name: plain_tag_0.name.clone().unwrap(),
                    r#type: plain_tag_0.r#type.clone(),
                    created_at: plain_tag_0.created_at,
                },
                TagVisible {
                    id: plain_tag_2.id,
                    name: plain_tag_2.name.unwrap(),
                    r#type: plain_tag_2.r#type,
                    created_at: plain_tag_2.created_at,
                },
            ],
        ))),
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((
            tagged_diary,
            vec![TagVisible {
                id: plain_tag_0.id,
                name: plain_tag_0.name.unwrap(),
                r#type: plain_tag_0.r#type,
                created_at: plain_tag_0.created_at,
            }],
        ))),
        JournalVisibleWithTags::from(ReadingNoteVisibleWithTags::from((
            tagged_reading_note,
            vec![TagVisible::from(TagVisible {
                id: plain_tag_1.id,
                name: plain_tag_1.name.unwrap(),
                r#type: plain_tag_1.r#type,
                created_at: plain_tag_1.created_at,
            })],
        ))),
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
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri("/api/journals").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
