use crate::entities::{action, ambition, objective, tag, user};
use chrono::Utc;
use sea_orm::{prelude::*, DbConn, DbErr, Set};

#[cfg(test)]
pub async fn create_user(db: &DbConn, is_active: bool) -> Result<user::Model, DbErr> {
    use crate::entities::sea_orm_active_enums::TimezoneEnum;

    user::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(format!("{}@test.com", uuid::Uuid::new_v4().to_string())),
        password: Set("password".to_string()),
        first_name: Set("Lynx".to_string()),
        last_name: Set("Levin".to_string()),
        timezone: Set(TimezoneEnum::AsiaTokyo),
        is_active: Set(is_active),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn create_active_user(db: &DbConn) -> Result<user::Model, DbErr> {
    create_user(db, true).await
}

#[cfg(test)]
pub async fn create_inactive_user(db: &DbConn) -> Result<user::Model, DbErr> {
    create_user(db, false).await
}

#[cfg(test)]
pub async fn create_tag(
    db: &DbConn,
    ambition_id: Option<uuid::Uuid>,
    objective_id: Option<uuid::Uuid>,
    action_id: Option<uuid::Uuid>,
    user_id: uuid::Uuid,
) -> Result<tag::Model, DbErr> {
    tag::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        user_id: Set(user_id),
        ambition_id: Set(ambition_id),
        objective_id: Set(objective_id),
        action_id: Set(action_id),
        created_at: Set(Utc::now().into()),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn create_ambition_and_tag(
    db: &DbConn,
    name: String,
    description: Option<String>,
    user_id: uuid::Uuid,
) -> Result<(ambition::Model, tag::Model), DbErr> {
    let ambition = ambition::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(name),
        description: Set(description),
        user_id: Set(user_id),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(db)
    .await?;

    let tag = create_tag(db, Some(ambition.id), None, None, user_id).await?;

    Ok((ambition, tag))
}

#[cfg(test)]
pub async fn create_objective_and_tag(
    db: &DbConn,
    name: String,
    user_id: uuid::Uuid,
) -> Result<(objective::Model, tag::Model), DbErr> {
    let objective = objective::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(name),
        user_id: Set(user_id),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(db)
    .await?;

    let tag = create_tag(db, None, Some(objective.id), None, user_id).await?;

    Ok((objective, tag))
}

#[cfg(test)]
pub async fn create_action_and_tag(
    db: &DbConn,
    action_name: String,
    user_id: uuid::Uuid,
) -> Result<(action::Model, tag::Model), DbErr> {
    let action = action::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(action_name),
        user_id: Set(user_id),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(db)
    .await?;

    let tag = create_tag(db, None, None, Some(action.id), user_id).await?;

    Ok((action, tag))
}
