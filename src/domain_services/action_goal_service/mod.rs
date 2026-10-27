use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod action_goal_mutation;

pub use action_goal_mutation::*;

#[derive(Debug, Error)]
pub enum ActionGoalServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("Action cannot be found.")]
    ActionNotFound(),
    #[error("ActionGoal cannot be found.")]
    ActionGoalNotFound(),
    #[error("For TimeSpan-type action, duration_seconds must be set.")]
    DurationSecondsNullForTimeSpanAction(),
    #[error("For TimeSpan-type action, count cannot be set.")]
    CountNotNullForTimeSpanAction(),
    #[error("For Count-type action, count must be set.")]
    CountNullForCountAction(),
    #[error("For Count-type action, duration_seconds cannot be set.")]
    DurationSecondsNotNullForCountAction(),
}
impl From<TransactionError<ActionGoalServiceError>> for ActionGoalServiceError {
    fn from(value: TransactionError<ActionGoalServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

#[derive(Clone)]
pub struct ActionGoalService<'a> {
    pub db: &'a DbConn,
}

impl<'a> ActionGoalService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
