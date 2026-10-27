use chrono::{DateTime, FixedOffset};
use sea_orm::DerivePartialModel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::{
    action::{self, ActionTrackType},
    action_goal,
    prelude::Action,
};

use crate::my_way::action_goals::types::ActionGoalVisible;

#[derive(Serialize, Deserialize, DerivePartialModel, PartialEq, Debug)]
#[sea_orm(entity = "Action")]
pub struct ActionVisible {
    pub id: Uuid,
    pub name: String,
    pub discipline: Option<String>,
    pub memo: Option<String>,
    pub color: String,
    pub track_type: ActionTrackType,
    pub archived: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<&action::Model> for ActionVisible {
    fn from(item: &action::Model) -> Self {
        ActionVisible {
            id: item.id,
            name: item.name.clone(),
            discipline: item.discipline.clone(),
            memo: item.memo.clone(),
            color: item.color.clone(),
            track_type: item.track_type.clone(),
            archived: item.archived,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

impl From<action::Model> for ActionVisible {
    fn from(item: action::Model) -> Self {
        ActionVisible::from(&item)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ActionVisibleWithGoal {
    pub id: Uuid,
    pub name: String,
    pub discipline: Option<String>,
    pub memo: Option<String>,
    pub color: String,
    pub track_type: ActionTrackType,
    pub archived: bool,
    pub goal: Option<ActionGoalVisible>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<&action::Model> for ActionVisibleWithGoal {
    fn from(value: &action::Model) -> Self {
        ActionVisibleWithGoal {
            id: value.id,
            name: value.name.clone(),
            discipline: value.discipline.clone(),
            memo: value.memo.clone(),
            color: value.color.clone(),
            track_type: value.track_type.clone(),
            archived: value.archived,
            goal: None,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
impl From<action::Model> for ActionVisibleWithGoal {
    fn from(value: action::Model) -> Self {
        ActionVisibleWithGoal::from(&value)
    }
}
impl ActionVisibleWithGoal {
    pub fn with_goal(mut self, goal: Option<action_goal::Model>) -> Self {
        self.goal = match goal {
            Some(goal) => Some(ActionGoalVisible::from(goal)),
            None => None,
        };
        self
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionCreateRequest {
    pub name: String,
    pub discipline: Option<String>,
    pub memo: Option<String>,
    pub color: Option<String>,
    pub track_type: ActionTrackType,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionUpdateRequest {
    pub name: String,
    pub discipline: Option<String>,
    pub memo: Option<String>,
    pub color: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionBulkUpdateOrderRequest {
    pub ordering: Vec<uuid::Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionTrackTypeConversionRequest {
    pub track_type: ActionTrackType,
}
