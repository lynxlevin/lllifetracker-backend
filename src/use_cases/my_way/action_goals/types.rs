use chrono::NaiveDate;
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::{action_goal, prelude::ActionGoal};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "ActionGoal")]
pub struct ActionGoalVisible {
    pub id: Uuid,
    pub from_date: NaiveDate,
    pub to_date: Option<NaiveDate>,
    pub duration_seconds: Option<i32>,
    pub count: Option<i32>,
}

impl From<&action_goal::Model> for ActionGoalVisible {
    fn from(value: &action_goal::Model) -> Self {
        ActionGoalVisible {
            id: value.id,
            from_date: value.from_date,
            to_date: value.to_date,
            duration_seconds: value.duration_seconds,
            count: value.count,
        }
    }
}

impl From<action_goal::Model> for ActionGoalVisible {
    fn from(value: action_goal::Model) -> Self {
        ActionGoalVisible::from(&value)
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionGoalSetNewRequest {
    pub action_id: Uuid,
    pub duration_seconds: Option<i32>,
    pub count: Option<i32>,
}
