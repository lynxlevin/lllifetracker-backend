use crate::entities::{action, tag, user};
use chrono::Utc;
use sea_orm::{prelude::*, DbConn, DbErr, Set, TransactionTrait};

#[cfg(test)]
pub async fn get_or_create_user(db: &DbConn) -> Result<user::Model, DbErr> {
    use crate::entities::sea_orm_active_enums::TimezoneEnum;

    match user::Entity::find()
        .filter(user::Column::Email.eq("test@test.com".to_string()))
        .one(db)
        .await?
    {
        Some(user) => Ok(user),
        None => Ok(user::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            email: Set(format!("{}@test.com", uuid::Uuid::new_v4().to_string())),
            password: Set("password".to_string()),
            first_name: Set("Lynx".to_string()),
            last_name: Set("Levin".to_string()),
            timezone: Set(TimezoneEnum::AsiaTokyo),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        }
        .insert(db)
        .await?),
    }
}

#[cfg(test)]
pub async fn get_or_create_action_and_tag(
    db: &DbConn,
    action_name: String,
    user_id: uuid::Uuid,
) -> (action::Model, tag::Model) {
    db.transaction::<_, (action::Model, tag::Model), DbErr>(|txn| {
        Box::pin(async move {
            let action = match action::Entity::find()
                .filter(action::Column::Name.eq(action_name.clone()))
                .filter(action::Column::UserId.eq(user_id))
                .one(txn)
                .await?
            {
                Some(action) => action,
                None => {
                    action::ActiveModel {
                        id: Set(uuid::Uuid::new_v4()),
                        name: Set(action_name),
                        user_id: Set(user_id),
                        created_at: Set(Utc::now().into()),
                        updated_at: Set(Utc::now().into()),
                    }
                    .insert(txn)
                    .await?
                }
            };

            let tag = match tag::Entity::find()
                .filter(tag::Column::ActionId.eq(action.id))
                .filter(tag::Column::UserId.eq(user_id))
                .one(txn)
                .await?
            {
                Some(tag) => tag,
                None => {
                    tag::ActiveModel {
                        id: Set(uuid::Uuid::new_v4()),
                        user_id: Set(user_id),
                        ambition_id: Set(None),
                        objective_id: Set(None),
                        action_id: Set(Some(action.id)),
                        created_at: Set(Utc::now().into()),
                    }
                    .insert(txn)
                    .await?
                }
            };
            Ok((action, tag))
        })
    })
    .await
    .unwrap()
}
