use crate::entities::action;
// use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::QueryOrder;

// #[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
// pub struct NewUser {
//     pub email: String,
//     pub password: String,
//     pub first_name: String,
//     pub last_name: String,
//     pub is_active: bool,
// }

// pub struct Mutation;

// impl Mutation {
//     pub async fn create_user(db: &DbConn, form_data: NewUser) -> Result<user::Model, DbErr> {
//         user::ActiveModel {
//             id: Set(uuid::Uuid::new_v4()),
//             email: Set(form_data.email.to_owned()),
//             password: Set(form_data.password.to_owned()),
//             first_name: Set(form_data.first_name.to_owned()),
//             last_name: Set(form_data.last_name.to_owned()),
//             is_active: Set(form_data.is_active.to_owned()),
//             ..Default::default()
//         }
//         .insert(db)
//         .await
//     }
// }

pub struct Query;

impl Query {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<action::Model>, DbErr> {
        action::Entity::find()
            .filter(action::Column::UserId.eq(user_id))
            .order_by_asc(action::Column::CreatedAt)
            .all(db)
            .await
    }
}
