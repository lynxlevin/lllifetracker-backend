use crate::entities::action_track;
use chrono::{Duration, Utc};
use sea_orm::{ActiveValue::NotSet, Set};
use uuid::Uuid;

#[cfg(test)]
pub fn action_track(user_id: Uuid) -> action_track::ActiveModel {
    action_track::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        action_id: NotSet,
        started_at: Set(Utc::now().into()),
        ended_at: NotSet,
        duration: NotSet,
    }
}

#[cfg(test)]
pub trait ActionTrackFactory {
    fn action_id(self, action_id: Option<Uuid>) -> action_track::ActiveModel;
    fn duration(self, duration: Option<i64>) -> action_track::ActiveModel;
}

#[cfg(test)]
impl ActionTrackFactory for action_track::ActiveModel {
    fn action_id(mut self, action_id: Option<Uuid>) -> action_track::ActiveModel {
        self.action_id = Set(action_id);
        self
    }

    fn duration(mut self, duration: Option<i64>) -> action_track::ActiveModel {
        self.duration = Set(duration);
        match duration {
            Some(duration) => {
                let now = Utc::now();
                self.ended_at = Set(Some(now.into()));
                self.started_at = Set((now - Duration::seconds(duration)).into());
            }
            None => self.ended_at = Set(None),
        }
        self
    }
}
