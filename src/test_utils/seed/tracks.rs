use crate::entities::action_track;
use chrono::{Duration, Utc};
use sea_orm::{prelude::*, DbConn, DbErr, Set};

#[cfg(test)]
pub async fn create_action_track(
    db: &DbConn,
    duration: Option<i64>,
    action_id: Option<uuid::Uuid>,
    user_id: uuid::Uuid,
) -> Result<action_track::Model, DbErr> {
    match duration {
        Some(duration) => {
            let now = Utc::now();
            action_track::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                user_id: Set(user_id),
                action_id: Set(action_id),
                started_at: Set((now - Duration::seconds(duration.into())).into()),
                ended_at: Set(Some(now.into())),
                duration: Set(Some(duration)),
            }
            .insert(db)
            .await
        },
        None => {
            let now = Utc::now();
            action_track::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                user_id: Set(user_id),
                action_id: Set(action_id),
                started_at: Set(now.into()),
                ended_at: Set(None),
                duration: Set(None),
            }
            .insert(db)
            .await
        }
    }
}
