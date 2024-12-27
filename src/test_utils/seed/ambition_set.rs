use crate::entities::{action, ambition, objective, tag};
use chrono::Utc;
use sea_orm::{prelude::*, DbConn, DbErr, Set};

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
    let now = Utc::now();
    let ambition = ambition::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(name),
        description: Set(description),
        archived: Set(false),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
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
    description: Option<String>,
    user_id: uuid::Uuid,
) -> Result<(objective::Model, tag::Model), DbErr> {
    let now = Utc::now();
    let objective = objective::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(name),
        description: Set(description),
        archived: Set(false),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(db)
    .await?;

    let tag = create_tag(db, None, Some(objective.id), None, user_id).await?;

    Ok((objective, tag))
}

#[cfg(test)]
pub async fn create_action(
    db: &DbConn,
    name: String,
    description: Option<String>,
    user_id: uuid::Uuid,
) -> Result<action::Model, DbErr> {
    let now = Utc::now();
    action::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(name),
        description: Set(description),
        archived: Set(false),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn create_action_and_tag(
    db: &DbConn,
    name: String,
    description: Option<String>,
    user_id: uuid::Uuid,
) -> Result<(action::Model, tag::Model), DbErr> {
    let action = create_action(db, name, description, user_id).await?;
    let tag = create_tag(db, None, None, Some(action.id), user_id).await?;
    Ok((action, tag))
}

#[cfg(test)]
pub async fn create_set_of_ambition_objective_action(
    db: &DbConn,
    user_id: uuid::Uuid,
    connect_ambition_objective: bool,
    connect_objective_action: bool,
) -> Result<(ambition::Model, objective::Model, action::Model), DbErr> {
    let (ambition, _) = create_ambition_and_tag(
        db,
        "ambition".to_string(),
        Some("Ambition".to_string()),
        user_id,
    )
    .await?;
    let (objective, _) = create_objective_and_tag(
        db,
        "objective".to_string(),
        Some("Objective".to_string()),
        user_id,
    )
    .await?;
    let (action, _) = create_action_and_tag(
        db,
        "action".to_string(),
        Some("Action".to_string()),
        user_id,
    )
    .await?;
    if connect_ambition_objective {
        ambition.clone().connect_objective(db, objective.id).await?;
    }
    if connect_objective_action {
        objective.clone().connect_action(db, action.id).await?;
    }

    Ok((ambition, objective, action))
}
