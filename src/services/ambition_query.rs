use crate::entities::{action, ambition, ambitions_objectives, objective, objectives_actions};
use crate::types::{AmbitionVisibleWithLinks, CustomDbErr};
use sea_orm::entity::prelude::*;
use sea_orm::{JoinType::LeftJoin, QueryOrder, QuerySelect};

pub struct AmbitionQuery;

impl AmbitionQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ambition::Model>, DbErr> {
        ambition::Entity::find()
            .filter(ambition::Column::UserId.eq(user_id))
            .order_by_asc(ambition::Column::CreatedAt)
            .all(db)
            .await
    }

    pub async fn find_all_with_linked_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<AmbitionVisibleWithLinks>, DbErr> {
        ambition::Entity::find()
            .filter(ambition::Column::UserId.eq(user_id))
            .column_as(objective::Column::Id, "objective_id")
            .column_as(objective::Column::Name, "objective_name")
            .column_as(objective::Column::CreatedAt, "objective_created_at")
            .column_as(objective::Column::UpdatedAt, "objective_updated_at")
            .column_as(action::Column::Id, "action_id")
            .column_as(action::Column::Name, "action_name")
            .column_as(action::Column::CreatedAt, "action_created_at")
            .column_as(action::Column::UpdatedAt, "action_updated_at")
            .join_rev(LeftJoin, ambitions_objectives::Relation::Ambition.def())
            .join(LeftJoin, ambitions_objectives::Relation::Objective.def())
            .join_rev(LeftJoin, objectives_actions::Relation::Objective.def())
            .join(LeftJoin, objectives_actions::Relation::Action.def())
            .order_by_asc(ambition::Column::Id)
            .order_by_asc(objective::Column::Id)
            .order_by_asc(action::Column::Id)
            .into_model::<AmbitionVisibleWithLinks>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<ambition::Model, DbErr> {
        ambition::Entity::find_by_id(ambition_id)
            .filter(ambition::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::test_utils;

//     use super::*;

//     #[actix_web::test]
//     async fn create_with_tag() -> Result<(), DbErr> {
//         let db = test_utils::init_db().await?;
//         let user = test_utils::seed::create_active_user(&db).await?;
//         let name = "Test ambition_service::Mutation::create_with_tag".to_string();
//         let description = Some("Dummy description".to_string());

//         let form_data = NewAmbition {
//             name: name.clone(),
//             description: description.clone(),
//             user_id: user.id,
//         };

//         let returned_ambition = Mutation::create_with_tag(&db, form_data).await.unwrap();
//         assert_eq!(returned_ambition.name, name);
//         assert_eq!(returned_ambition.description, description);
//         assert_eq!(returned_ambition.user_id, user.id);

//         let created_ambition = ambition::Entity::find_by_id(returned_ambition.id)
//             .filter(ambition::Column::Name.eq(name))
//             .filter(ambition::Column::Description.eq(description))
//             .filter(ambition::Column::UserId.eq(user.id))
//             .filter(ambition::Column::CreatedAt.eq(returned_ambition.created_at))
//             .filter(ambition::Column::UpdatedAt.eq(returned_ambition.updated_at))
//             .one(&db)
//             .await?;
//         assert!(created_ambition.is_some());

//         let created_tag = tag::Entity::find()
//             .filter(tag::Column::UserId.eq(user.id))
//             .filter(tag::Column::AmbitionId.eq(Some(returned_ambition.id)))
//             .filter(tag::Column::ObjectiveId.is_null())
//             .filter(tag::Column::ActionId.is_null())
//             .one(&db)
//             .await?;
//         assert!(created_tag.is_some());

//         Ok(())
//     }
// }
