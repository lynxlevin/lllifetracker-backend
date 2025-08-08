use chrono::{NaiveDate, Utc};
use entities::action_goal;
use sea_orm::{ActiveValue::NotSet, Set};
use uuid::Uuid;

pub fn action_goal(user_id: Uuid, action_id: Uuid) -> action_goal::ActiveModel {
    let now = Utc::now();
    action_goal::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        action_id: Set(action_id),
        from_date: Set(now.date_naive()),
        to_date: NotSet,
        duration_seconds: NotSet,
        count: NotSet,
    }
}

pub trait ActionGoalFactory {
    fn from_date(self, from_date: NaiveDate) -> action_goal::ActiveModel;
    fn to_date(self, to_date: Option<NaiveDate>) -> action_goal::ActiveModel;
    fn duration_seconds(self, duration_seconds: Option<i32>) -> action_goal::ActiveModel;
    fn count(self, count: Option<i32>) -> action_goal::ActiveModel;
}

impl ActionGoalFactory for action_goal::ActiveModel {
    fn from_date(mut self, from_date: NaiveDate) -> action_goal::ActiveModel {
        self.from_date = Set(from_date);
        self
    }
    fn to_date(mut self, to_date: Option<NaiveDate>) -> action_goal::ActiveModel {
        self.to_date = Set(to_date);
        self
    }
    fn duration_seconds(mut self, duration_seconds: Option<i32>) -> action_goal::ActiveModel {
        self.duration_seconds = Set(duration_seconds);
        self
    }
    fn count(mut self, count: Option<i32>) -> action_goal::ActiveModel {
        self.count = Set(count);
        self
    }
}
