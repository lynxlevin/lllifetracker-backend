use actix_web::{
    http,
    test::{self, TestRequest},
    HttpMessage,
};
use sea_orm::{ActiveModelTrait, DbErr};

use crate::utils::{init_app, Connections};
use common::factory::{self, create_tags, DiaryFactory, ReadingNoteFactory, TagParam, ThinkingNoteFactory};
use use_cases::{
    journal::{
        diaries::types::DiaryVisibleWithTags,
        reading_notes::types::ReadingNoteVisibleWithTags,
        thinking_notes::types::ThinkingNoteVisibleWithTags,
        types::{JournalSearchRequest, JournalVisibleWithTags},
    },
    tags::types::TagVisible,
};

const URI: &str = "/api/journals/search";
fn get_client() -> TestRequest {
    test::TestRequest::post()
}

#[actix_web::test]
async fn texts_should_be_space_separated_and_condition() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let search_text = "Find me";
    let hit_0 = factory::diary(user.id)
        .text(Some("Find me".to_string()))
        .insert(&db)
        .await?;
    let hit_1 = factory::diary(user.id)
        .text(Some("Findme".to_string()))
        .insert(&db)
        .await?;
    let hit_2 = factory::diary(user.id)
        .text(Some("me Find".to_string()))
        .insert(&db)
        .await?;
    let hit_3 = factory::diary(user.id)
        .text(Some("xFind mex".to_string()))
        .insert(&db)
        .await?;
    let hit_4 = factory::reading_note(user.id)
        .title("Find".to_string())
        .text("me".to_string())
        .insert(&db)
        .await?;
    let _no_hit_0 = factory::diary(user.id)
        .text(Some("find me".to_string()))
        .insert(&db)
        .await?;
    let _no_hit_1 = factory::diary(user.id)
        .text(Some("Find".to_string()))
        .insert(&db)
        .await?;

    let req = get_client()
        .uri(URI)
        .set_json(JournalSearchRequest { text: Some(search_text.to_string()), tag_ids: vec![] })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<JournalVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((hit_3, vec![]))),
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((hit_2, vec![]))),
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((hit_1, vec![]))),
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((hit_0, vec![]))),
        JournalVisibleWithTags::from(ReadingNoteVisibleWithTags::from((hit_4, vec![]))),
    ];

    assert_eq!(body.len(), expected.len());
    for i in 0..body.len() {
        dbg!(i);
        assert_eq!(body[i], expected[i]);
    }

    Ok(())
}

#[actix_web::test]
async fn tags_should_be_or_condition() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let tags = create_tags(
        vec![
            TagParam { name: "tag_0", ..Default::default() },
            TagParam { name: "tag_1", ..Default::default() },
            TagParam { name: "_no_hit_tag", ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let tag_0 = tags.get("tag_0").unwrap();
    let tag_1 = tags.get("tag_1").unwrap();

    let hit_diary_0 = factory::diary(user.id).insert(&db).await?;
    factory::link_diary_tag(&db, hit_diary_0.id, tag_0.id).await?;
    let hit_diary_1 = factory::diary(user.id).insert(&db).await?;
    factory::link_diary_tag(&db, hit_diary_1.id, tag_1.id).await?;
    let no_hit_diary = factory::diary(user.id).insert(&db).await?;
    factory::link_diary_tag(&db, no_hit_diary.id, tags.get("_no_hit_tag").unwrap().id).await?;

    let hit_reading_note_0 = factory::reading_note(user.id).insert(&db).await?;
    factory::link_reading_note_tag(&db, hit_reading_note_0.id, tag_0.id).await?;
    let hit_reading_note_1 = factory::reading_note(user.id).insert(&db).await?;
    factory::link_reading_note_tag(&db, hit_reading_note_1.id, tag_1.id).await?;
    let no_hit_reading_note = factory::reading_note(user.id).insert(&db).await?;
    factory::link_reading_note_tag(&db, no_hit_reading_note.id, tags.get("_no_hit_tag").unwrap().id).await?;

    let hit_thinking_note_0 = factory::thinking_note(user.id).insert(&db).await?;
    factory::link_thinking_note_tag(&db, hit_thinking_note_0.id, tag_0.id).await?;
    let hit_thinking_note_1 = factory::thinking_note(user.id).insert(&db).await?;
    factory::link_thinking_note_tag(&db, hit_thinking_note_1.id, tag_1.id).await?;
    let no_hit_thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    factory::link_thinking_note_tag(&db, no_hit_thinking_note.id, tags.get("_no_hit_tag").unwrap().id).await?;

    let req = get_client()
        .uri(URI)
        .set_json(JournalSearchRequest { text: None, tag_ids: vec![tag_0.id, tag_1.id] })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<JournalVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((
            hit_diary_1,
            vec![TagVisible::from((tag_1, tag_1.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((
            hit_diary_0,
            vec![TagVisible::from((tag_0, tag_0.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(ReadingNoteVisibleWithTags::from((
            hit_reading_note_1,
            vec![TagVisible::from((tag_1, tag_1.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(ReadingNoteVisibleWithTags::from((
            hit_reading_note_0,
            vec![TagVisible::from((tag_0, tag_0.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((
            hit_thinking_note_1,
            vec![TagVisible::from((tag_1, tag_1.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((
            hit_thinking_note_0,
            vec![TagVisible::from((tag_0, tag_0.name.clone().unwrap()))],
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
async fn text_and_tag_combination_should_be_and_condition() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let tag = factory::tag(user.id).insert(&db).await?;
    let search_text = "Find me";

    let hit_diary = factory::diary(user.id)
        .text(Some(search_text.to_string()))
        .insert(&db)
        .await?;
    factory::link_diary_tag(&db, hit_diary.id, tag.id).await?;
    let _no_hit_text_only_diary = factory::diary(user.id)
        .text(Some(search_text.to_string()))
        .insert(&db)
        .await?;
    let no_hit_tag_only_diary = factory::diary(user.id).insert(&db).await?;
    factory::link_diary_tag(&db, no_hit_tag_only_diary.id, tag.id).await?;

    let hit_reading_note = factory::reading_note(user.id)
        .text(search_text.to_string())
        .insert(&db)
        .await?;
    factory::link_reading_note_tag(&db, hit_reading_note.id, tag.id).await?;
    let _no_hit_text_only_reading_note = factory::reading_note(user.id)
        .text(search_text.to_string())
        .insert(&db)
        .await?;
    let no_hit_tag_only_reading_note = factory::reading_note(user.id).insert(&db).await?;
    factory::link_reading_note_tag(&db, no_hit_tag_only_reading_note.id, tag.id).await?;

    let hit_thinking_note = factory::thinking_note(user.id)
        .question(Some(search_text.to_string()))
        .insert(&db)
        .await?;
    factory::link_thinking_note_tag(&db, hit_thinking_note.id, tag.id).await?;
    let _no_hit_text_only_thinking_note = factory::thinking_note(user.id)
        .question(Some(search_text.to_string()))
        .insert(&db)
        .await?;
    let no_hit_tag_only_thinking_note = factory::thinking_note(user.id).insert(&db).await?;
    factory::link_thinking_note_tag(&db, no_hit_tag_only_thinking_note.id, tag.id).await?;

    let req = get_client()
        .uri(URI)
        .set_json(JournalSearchRequest { text: Some(search_text.to_string()), tag_ids: vec![tag.id] })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<JournalVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((
            hit_diary,
            vec![TagVisible::from((&tag, tag.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(ReadingNoteVisibleWithTags::from((
            hit_reading_note,
            vec![TagVisible::from((&tag, tag.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((
            hit_thinking_note,
            vec![TagVisible::from((&tag, tag.name.clone().unwrap()))],
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

    let req = get_client()
        .uri(URI)
        .set_json(JournalSearchRequest { text: None, tag_ids: vec![] })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
