use actix_web::{http, test, HttpMessage};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, DeriveColumn, EntityTrait, EnumIter, QueryFilter,
    QuerySelect,
};
use use_cases::journal::diaries::types::{DiaryCreateRequest, DiaryVisible};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{diaries_tags, diary};

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let (_, tag_0) = factory::action(user.id)
        .name("action_0".to_string())
        .insert_with_tag(&db)
        .await?;
    let (_, tag_1) = factory::action(user.id)
        .name("action_1".to_string())
        .insert_with_tag(&db)
        .await?;

    let diary_text = Some("This is a new diary for testing create method.".to_string());
    let today = chrono::Utc::now().date_naive();
    let diary_score = Some(2);
    let req = test::TestRequest::post()
        .uri("/api/diaries")
        .set_json(DiaryCreateRequest {
            text: diary_text.clone(),
            date: today,
            score: diary_score,
            tag_ids: vec![tag_0.id, tag_1.id],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: DiaryVisible = test::read_body_json(res).await;
    assert_eq!(res.text, diary_text.clone());
    assert_eq!(res.date, today);

    let diary_in_db = diary::Entity::find_by_id(res.id).one(&db).await?.unwrap();
    assert_eq!(diary_in_db.user_id, user.id);
    assert_eq!(DiaryVisible::from(diary_in_db), res);

    let linked_tag_ids: Vec<uuid::Uuid> = diaries_tags::Entity::find()
        .column_as(diaries_tags::Column::TagId, QueryAs::TagId)
        .filter(diaries_tags::Column::DiaryId.eq(res.id))
        .into_values::<_, QueryAs>()
        .all(&db)
        .await?;
    assert_eq!(linked_tag_ids.len(), 2);
    assert!(linked_tag_ids.contains(&tag_0.id));
    assert!(linked_tag_ids.contains(&tag_1.id));

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/diaries")
        .set_json(DiaryCreateRequest {
            text: None,
            date: chrono::Utc::now().date_naive(),
            score: None,
            tag_ids: vec![],
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn conflict_if_duplicate_exists() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _existing_diary = factory::diary(user.id)
        .date(chrono::NaiveDate::from_ymd_opt(2025, 3, 19).unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::post()
        .uri("/api/diaries")
        .set_json(DiaryCreateRequest {
            text: None,
            date: chrono::NaiveDate::from_ymd_opt(2025, 3, 19).unwrap(),
            score: None,
            tag_ids: vec![],
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CONFLICT);

    Ok(())
}

#[actix_web::test]
async fn not_found_on_non_existent_tag_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let today = chrono::Utc::now().date_naive();

    let non_existent_tag_req = test::TestRequest::post()
        .uri("/api/diaries")
        .set_json(DiaryCreateRequest {
            text: None,
            date: today,
            score: None,
            tag_ids: vec![uuid::Uuid::now_v7()],
        })
        .to_request();
    non_existent_tag_req.extensions_mut().insert(user.clone());
    let res = test::call_service(&app, non_existent_tag_req).await;
    assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

mod validation_errors {
    use super::*;

    #[actix_web::test]
    async fn too_large_score() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let today = chrono::Utc::now().date_naive();

        let score_too_large_req = test::TestRequest::post()
            .uri("/api/diaries")
            .set_json(DiaryCreateRequest {
                text: None,
                date: today,
                score: Some(6),
                tag_ids: vec![],
            })
            .to_request();
        score_too_large_req.extensions_mut().insert(user.clone());
        let res = test::call_service(&app, score_too_large_req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[actix_web::test]
    async fn too_small_score() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let today = chrono::Utc::now().date_naive();

        let score_too_small_req = test::TestRequest::post()
            .uri("/api/diaries")
            .set_json(DiaryCreateRequest {
                text: None,
                date: today,
                score: Some(0),
                tag_ids: vec![],
            })
            .to_request();
        score_too_small_req.extensions_mut().insert(user.clone());
        let res = test::call_service(&app, score_too_small_req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }
}
