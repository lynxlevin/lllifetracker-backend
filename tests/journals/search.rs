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
async fn happy_path() -> Result<(), DbErr> {
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
    let search_text = "Find me";

    let text_hit_diary = factory::diary(user.id)
        .text(Some(search_text.to_string()))
        .insert(&db)
        .await?;
    let title_hit_reading_note = factory::reading_note(user.id)
        .title(search_text.to_string())
        .insert(&db)
        .await?;
    let text_hit_reading_note = factory::reading_note(user.id)
        .text(search_text.to_string())
        .insert(&db)
        .await?;
    let question_hit_thinking_note = factory::thinking_note(user.id)
        .question(Some(search_text.to_string()))
        .insert(&db)
        .await?;
    let thought_hit_thinking_note = factory::thinking_note(user.id)
        .thought(Some(search_text.to_string()))
        .insert(&db)
        .await?;
    let answer_hit_thinking_note = factory::thinking_note(user.id)
        .answer(Some(search_text.to_string()))
        .insert(&db)
        .await?;
    let tag_hit_diary_0 = factory::diary(user.id).insert(&db).await?;
    factory::link_diary_tag(&db, tag_hit_diary_0.id, tag_0.id).await?;
    let tag_hit_diary_1 = factory::diary(user.id).insert(&db).await?;
    factory::link_diary_tag(&db, tag_hit_diary_1.id, tag_1.id).await?;
    let no_hit_diary = factory::diary(user.id).insert(&db).await?;
    factory::link_diary_tag(&db, no_hit_diary.id, tags.get("_no_hit_tag").unwrap().id).await?;

    let req = get_client()
        .uri(URI)
        .set_json(JournalSearchRequest { text: Some(search_text.to_string()), tag_ids: vec![tag_0.id, tag_1.id] })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<JournalVisibleWithTags> = test::read_body_json(resp).await;
    let expected = vec![
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((
            tag_hit_diary_1,
            vec![TagVisible::from((tag_1, tag_1.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((
            tag_hit_diary_0,
            vec![TagVisible::from((tag_0, tag_0.name.clone().unwrap()))],
        ))),
        JournalVisibleWithTags::from(DiaryVisibleWithTags::from((text_hit_diary, vec![]))),
        JournalVisibleWithTags::from(ReadingNoteVisibleWithTags::from((text_hit_reading_note, vec![]))),
        JournalVisibleWithTags::from(ReadingNoteVisibleWithTags::from((title_hit_reading_note, vec![]))),
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((answer_hit_thinking_note, vec![]))),
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((thought_hit_thinking_note, vec![]))),
        JournalVisibleWithTags::from(ThinkingNoteVisibleWithTags::from((question_hit_thinking_note, vec![]))),
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
