use actix_web::{http, test, HttpMessage};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter,
    QueryFilter, QuerySelect,
};

use super::super::utils::init_app;
use entities::{diaries_tags, diary};
use test_utils::{self, *};
use types::*;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let diary = factory::diary(user.id).insert(&db).await?;
    let (_, tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    let form = DiaryUpdateRequest {
        text: None,
        date: chrono::NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
        score: None,
        tag_ids: vec![tag.id],
        update_keys: vec![
            DiaryKey::Text,
            DiaryKey::Date,
            DiaryKey::Score,
            DiaryKey::TagIds,
        ],
    };

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}", diary.id))
        .set_json(&form)
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: DiaryVisible = test::read_body_json(res).await;
    assert_eq!(res.text, form.text.clone());
    assert_eq!(res.date, form.date);
    assert_eq!(res.score, form.score);

    let diary_in_db = diary::Entity::find_by_id(res.id).one(&db).await?.unwrap();
    assert_eq!(diary_in_db.user_id, user.id);
    assert_eq!(DiaryVisible::from(diary_in_db), res);

    let linked_tag_ids: Vec<uuid::Uuid> = diaries_tags::Entity::find()
        .column_as(diaries_tags::Column::TagId, QueryAs::TagId)
        .filter(diaries_tags::Column::DiaryId.eq(res.id))
        .into_values::<_, QueryAs>()
        .all(&db)
        .await?;
    assert_eq!(linked_tag_ids.len(), 1);
    assert!(linked_tag_ids.contains(&tag.id));

    Ok(())
}

#[actix_web::test]
async fn not_found_if_invalid_id() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}", uuid::Uuid::now_v7()))
        .set_json(DiaryUpdateRequest {
            text: None,
            date: chrono::Utc::now().date_naive(),
            score: None,
            tag_ids: vec![],
            update_keys: vec![],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let diary = factory::diary(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}", diary.id))
        .set_json(DiaryUpdateRequest {
            text: None,
            date: chrono::Utc::now().date_naive(),
            score: None,
            tag_ids: vec![],
            update_keys: vec![],
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn conflict_if_duplicate_exists() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let diary = factory::diary(user.id).insert(&db).await?;
    let _existing_diary = factory::diary(user.id)
        .date(chrono::NaiveDate::from_ymd_opt(2025, 3, 19).unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}", diary.id))
        .set_json(DiaryUpdateRequest {
            text: None,
            date: chrono::NaiveDate::from_ymd_opt(2025, 3, 19).unwrap(),
            score: None,
            tag_ids: vec![],
            update_keys: vec![DiaryKey::Date],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CONFLICT);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_non_existent_tag_id() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let diary = factory::diary(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}", diary.id))
        .set_json(DiaryUpdateRequest {
            text: None,
            date: diary.date,
            score: None,
            tag_ids: vec![uuid::Uuid::now_v7()],
            update_keys: vec![DiaryKey::TagIds],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn validation_errors() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let diary = factory::diary(user.id).insert(&db).await?;

    let score_too_large_req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}", diary.id))
        .set_json(DiaryUpdateRequest {
            text: None,
            date: diary.date,
            score: Some(6),
            tag_ids: vec![],
            update_keys: vec![DiaryKey::Score],
        })
        .to_request();
    score_too_large_req.extensions_mut().insert(user.clone());
    let res = test::call_service(&app, score_too_large_req).await;
    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

    let score_too_small_req = test::TestRequest::put()
        .uri(&format!("/api/diaries/{}", diary.id))
        .set_json(DiaryUpdateRequest {
            text: None,
            date: diary.date,
            score: Some(0),
            tag_ids: vec![],
            update_keys: vec![DiaryKey::Score],
        })
        .to_request();
    score_too_small_req.extensions_mut().insert(user.clone());
    let res = test::call_service(&app, score_too_small_req).await;
    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

    Ok(())
}
