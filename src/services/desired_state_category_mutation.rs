use chrono::Utc;
use entities::{desired_state_category, mindset, tag};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DbConn, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter, Set, TransactionError, TransactionTrait,
};
use types::{CustomDbErr, DesiredStateConvertToType};
use uuid::Uuid;

use super::desired_state_query::DesiredStateQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewDesiredState {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub user_id: uuid::Uuid,
}

pub struct DesiredStateCategoryMutation;

impl DesiredStateCategoryMutation {
    pub async fn create(
        db: &DbConn,
        user_id: Uuid,
        name: String,
    ) -> Result<desired_state_category::Model, DbErr> {
        desired_state_category::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(user_id),
            name: Set(name),
            ordering: NotSet,
        }
        .insert(db)
        .await
    }

    // pub async fn update(
    //     db: &DbConn,
    //     desired_state_id: uuid::Uuid,
    //     user_id: uuid::Uuid,
    //     name: String,
    //     description: Option<String>,
    //     category_id: Option<Uuid>,
    // ) -> Result<desired_state_category::Model, DbErr> {
    //     let mut desired_state_category: desired_state_category::ActiveModel =
    //         DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
    //             .await?
    //             .into();
    //     desired_state_category.name = Set(name);
    //     desired_state_category.description = Set(description);
    //     desired_state_category.category_id = Set(category_id);
    //     desired_state_category.updated_at = Set(Utc::now().into());
    //     desired_state_category.update(db).await
    // }

    // pub async fn delete(
    //     db: &DbConn,
    //     desired_state_id: uuid::Uuid,
    //     user_id: uuid::Uuid,
    // ) -> Result<(), DbErr> {
    //     DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
    //         .await?
    //         .delete(db)
    //         .await?;
    //     Ok(())
    // }

    // // FIXME: Reduce query.
    // pub async fn bulk_update_ordering(
    //     db: &DbConn,
    //     user_id: uuid::Uuid,
    //     ordering: Vec<uuid::Uuid>,
    // ) -> Result<(), DbErr> {
    //     let desired_states = desired_state_category::Entity::find()
    //         .filter(desired_state_category::Column::UserId.eq(user_id))
    //         .filter(desired_state_category::Column::Id.is_in(ordering.clone()))
    //         .all(db)
    //         .await?;
    //     for desired_state_category in desired_states {
    //         let order = &ordering.iter().position(|id| id == &desired_state_category.id);
    //         if let Some(order) = order {
    //             let mut desired_state_category = desired_state_category.into_active_model();
    //             desired_state_category.ordering = Set(Some((order + 1) as i32));
    //             desired_state_category.update(db).await?;
    //         }
    //     }
    //     Ok(())
    // }
}

// #[cfg(test)]
// mod tests {
//     use ::types::CustomDbErr;
//     use common::{
//         db::init_db,
//         factory::{self, *},
//         settings::get_test_settings,
//     };
//     use types::DesiredStateConvertToType;

//     use super::*;

// #[actix_web::test]
// async fn update() -> Result<(), DbErr> {
//     let settings = get_test_settings();
//     let db = init_db(&settings).await;
//     let user = factory::user().insert(&db).await?;
//     let desired_state_category = factory::desired_state_category(user.id).insert(&db).await?;
//     let category = factory::desired_state_category(user.id).insert(&db).await?;

//     let new_name = "desired_state_after_update".to_string();
//     let new_description = "DesiredState after update.".to_string();

//     let res = DesiredStateCategoryMutation::update(
//         &db,
//         desired_state_category.id,
//         user.id,
//         new_name.clone(),
//         Some(new_description.clone()),
//         Some(category.id),
//     )
//     .await?;
//     assert_eq!(res.id, desired_state_category.id);
//     assert_eq!(res.name, new_name.clone());
//     assert_eq!(res.description, Some(new_description.clone()));
//     assert_eq!(res.category_id, Some(category.id));
//     assert_eq!(res.archived, desired_state_category.archived);
//     assert_eq!(res.user_id, user.id);
//     assert_eq!(res.created_at, desired_state_category.created_at);
//     assert!(res.updated_at > desired_state_category.updated_at);

//     let desired_state_in_db = desired_state_category::Entity::find_by_id(desired_state_category.id)
//         .one(&db)
//         .await?
//         .unwrap();
//     assert_eq!(desired_state_in_db, res);

//     Ok(())
// }

// #[actix_web::test]
// async fn update_unauthorized() -> Result<(), DbErr> {
//     let settings = get_test_settings();
//     let db = init_db(&settings).await;
//     let user = factory::user().insert(&db).await?;
//     let desired_state_category = factory::desired_state_category(user.id).insert(&db).await?;

//     let error = DesiredStateCategoryMutation::update(
//         &db,
//         desired_state_category.id,
//         uuid::Uuid::now_v7(),
//         "desired_state_after_update_unauthorized".to_string(),
//         None,
//         None,
//     )
//     .await
//     .unwrap_err();
//     assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

//     Ok(())
// }

// #[actix_web::test]
// async fn delete() -> Result<(), DbErr> {
//     let settings = get_test_settings();
//     let db = init_db(&settings).await;
//     let user = factory::user().insert(&db).await?;
//     let (desired_state_category, tag) = factory::desired_state_category(user.id).insert_with_tag(&db).await?;

//     DesiredStateCategoryMutation::delete(&db, desired_state_category.id, user.id).await?;

//     let desired_state_in_db = desired_state_category::Entity::find_by_id(desired_state_category.id)
//         .one(&db)
//         .await?;
//     assert!(desired_state_in_db.is_none());

//     let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
//     assert!(tag_in_db.is_none());

//     Ok(())
// }

// #[actix_web::test]
// async fn delete_unauthorized() -> Result<(), DbErr> {
//     let settings = get_test_settings();
//     let db = init_db(&settings).await;
//     let user = factory::user().insert(&db).await?;
//     let desired_state_category = factory::desired_state_category(user.id).insert(&db).await?;

//     let error = DesiredStateCategoryMutation::delete(&db, desired_state_category.id, uuid::Uuid::now_v7())
//         .await
//         .unwrap_err();
//     assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

//     Ok(())
// }

// #[actix_web::test]
// async fn bulk_update_ordering() -> Result<(), DbErr> {
//     let settings = get_test_settings();
//     let db = init_db(&settings).await;
//     let user = factory::user().insert(&db).await?;
//     let desired_state_0 = factory::desired_state_category(user.id).insert(&db).await?;
//     let desired_state_1 = factory::desired_state_category(user.id).insert(&db).await?;
//     let desired_state_2 = factory::desired_state_category(user.id).insert(&db).await?;

//     let ordering = vec![desired_state_0.id, desired_state_1.id];

//     DesiredStateCategoryMutation::bulk_update_ordering(&db, user.id, ordering).await?;

//     let desired_state_in_db_0 = desired_state_category::Entity::find_by_id(desired_state_0.id)
//         .one(&db)
//         .await?
//         .unwrap();
//     assert_eq!(desired_state_in_db_0.ordering, Some(1));

//     let desired_state_in_db_1 = desired_state_category::Entity::find_by_id(desired_state_1.id)
//         .one(&db)
//         .await?
//         .unwrap();
//     assert_eq!(desired_state_in_db_1.ordering, Some(2));

//     let desired_state_in_db_2 = desired_state_category::Entity::find_by_id(desired_state_2.id)
//         .one(&db)
//         .await?
//         .unwrap();
//     assert_eq!(desired_state_in_db_2.ordering, None);

//     Ok(())
// }

// #[actix_web::test]
// async fn bulk_update_ordering_no_modification_on_different_users_records() -> Result<(), DbErr>
// {
//     let settings = get_test_settings();
//     let db = init_db(&settings).await;
//     let user = factory::user().insert(&db).await?;
//     let another_user = factory::user().insert(&db).await?;
//     let another_users_desired_state =
//         factory::desired_state_category(another_user.id).insert(&db).await?;

//     let ordering = vec![another_users_desired_state.id];

//     DesiredStateCategoryMutation::bulk_update_ordering(&db, user.id, ordering).await?;

//     let another_users_desired_state_in_db =
//         desired_state_category::Entity::find_by_id(another_users_desired_state.id)
//             .one(&db)
//             .await?
//             .unwrap();
//     assert_eq!(another_users_desired_state_in_db.ordering, None);

//     Ok(())
// }
// }
