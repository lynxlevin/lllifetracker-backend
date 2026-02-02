use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::direction_categories::types::{
    DirectionCategoryUpdateRequest, DirectionCategoryVisible,
};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;
use entities::direction_category;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let category = factory::direction_category(user.id).insert(&db).await?;

    let new_name = "new name".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/api/direction_categories/{}", category.id))
        .set_json(DirectionCategoryUpdateRequest {
            name: new_name.clone(),
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: DirectionCategoryVisible = test::read_body_json(res).await;
    assert_eq!(res.id, category.id);
    assert_eq!(res.name, new_name);

    let category_in_db = direction_category::Entity::find_by_id(category.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(category_in_db.user_id, user.id);
    assert_eq!(category_in_db.ordering, None);
    assert_eq!(DirectionCategoryVisible::from(category_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/direction_categories/{}", Uuid::now_v7()))
        .set_json(DirectionCategoryUpdateRequest {
            name: String::default(),
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod not_found {
    use super::*;

    #[actix_web::test]
    async fn other_user_category() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let other_user = factory::user().insert(&db).await?;
        let other_user_category = factory::direction_category(other_user.id)
            .insert(&db)
            .await?;

        let req = test::TestRequest::put()
            .uri(&format!(
                "/api/direction_categories/{}",
                other_user_category.id
            ))
            .set_json(DirectionCategoryUpdateRequest {
                name: String::default(),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }
    #[actix_web::test]
    async fn non_existent_id() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;

        let req = test::TestRequest::put()
            .uri(&format!("/api/direction_categories/{}", Uuid::now_v7()))
            .set_json(DirectionCategoryUpdateRequest {
                name: String::default(),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }
}
