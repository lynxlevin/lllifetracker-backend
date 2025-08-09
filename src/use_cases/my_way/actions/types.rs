use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::{
    action, action_goal,
    prelude::{Action, ActionGoal},
    sea_orm_active_enums::ActionTrackType,
};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "Action")]
pub struct ActionVisible {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trackable: bool,
    pub color: String,
    pub track_type: ActionTrackType,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<&action::Model> for ActionVisible {
    fn from(item: &action::Model) -> Self {
        ActionVisible {
            id: item.id,
            name: item.name.clone(),
            description: item.description.clone(),
            trackable: item.trackable,
            color: item.color.clone(),
            track_type: item.track_type.clone(),
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ActionVisibleWithGoal {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trackable: bool,
    pub color: String,
    pub track_type: ActionTrackType,
    pub goal: Option<ActionGoalVisible>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<&(action::Model, Option<action_goal::Model>)> for ActionVisibleWithGoal {
    fn from(value: &(action::Model, Option<action_goal::Model>)) -> Self {
        ActionVisibleWithGoal {
            id: value.0.id,
            name: value.0.name.clone(),
            description: value.0.description.clone(),
            trackable: value.0.trackable,
            color: value.0.color.clone(),
            track_type: value.0.track_type.clone(),
            goal: value
                .1
                .as_ref()
                .and_then(|goal| Some(ActionGoalVisible::from(goal))),
            created_at: value.0.created_at,
            updated_at: value.0.updated_at,
        }
    }
}

impl From<(action::Model, Option<action_goal::Model>)> for ActionVisibleWithGoal {
    fn from(value: (action::Model, Option<action_goal::Model>)) -> Self {
        ActionVisibleWithGoal::from(&(value.0, value.1))
    }
}

#[derive(Deserialize, Debug)]
pub struct ActionListQuery {
    pub show_archived_only: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub track_type: ActionTrackType,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionUpdateRequest {
    pub name: String,
    pub description: Option<String>,
    pub trackable: Option<bool>,
    pub color: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionBulkUpdateOrderRequest {
    pub ordering: Vec<uuid::Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ActionTrackTypeConversionRequest {
    pub track_type: ActionTrackType,
}
