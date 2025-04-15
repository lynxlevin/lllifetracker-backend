use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use super::super::utils::init_app;
use entities::{diaries_tags, diary};
use test_utils::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let diary = factory::diary(user.id).insert(&db).await?;
    let (_, tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
    factory::link_diary_tag(&db, diary.id, tag.id).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/diaries/{}", diary.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let diary_in_db = diary::Entity::find_by_id(diary.id).one(&db).await?;
    assert!(diary_in_db.is_none());

    let diaries_tags_in_db = diaries_tags::Entity::find()
        .filter(diaries_tags::Column::DiaryId.eq(diary.id))
        .filter(diaries_tags::Column::TagId.eq(tag.id))
        .one(&db)
        .await?;
    assert!(diaries_tags_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let diary = factory::diary(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/diaries/{}", diary.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
