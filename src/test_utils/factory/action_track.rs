use entities::action_track;
use chrono::{Duration, SubsecRound, Utc};
use sea_orm::{ActiveValue::NotSet, Set};
use uuid::Uuid;

pub fn action_track(user_id: Uuid) -> action_track::ActiveModel {
    action_track::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        action_id: NotSet,
        started_at: Set(Utc::now().trunc_subsecs(0).into()),
        ended_at: NotSet,
        duration: NotSet,
    }
}

pub trait ActionTrackFactory {
    fn action_id(self, action_id: Option<Uuid>) -> action_track::ActiveModel;
    fn duration(self, duration: Option<i64>) -> action_track::ActiveModel;
    fn started_at(
        self,
        started_at: chrono::DateTime<chrono::FixedOffset>,
    ) -> action_track::ActiveModel;
}

impl ActionTrackFactory for action_track::ActiveModel {
    fn action_id(mut self, action_id: Option<Uuid>) -> action_track::ActiveModel {
        self.action_id = Set(action_id);
        self
    }

    fn duration(mut self, duration: Option<i64>) -> action_track::ActiveModel {
        self.duration = Set(duration);
        match duration {
            Some(duration) => {
                self.ended_at = Set(Some(
                    (self.started_at.clone().unwrap() + Duration::seconds(duration)).into(),
                ));
            }
            None => self.ended_at = Set(None),
        }
        self
    }

    fn started_at(
        mut self,
        started_at: chrono::DateTime<chrono::FixedOffset>,
    ) -> action_track::ActiveModel {
        self.started_at = Set(started_at);
        if self.duration == NotSet {
            return self;
        }
        if let Some(duration) = self.duration.clone().unwrap() {
            self.ended_at = Set((started_at + Duration::seconds(duration)).into());
        }
        self
    }
}
