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
    // #[error("ActionGoal cannot be found.")]
    // ActionGoalNotFound(),
    // #[error("Friend cannot be found.")]
    // FriendNotFound(),
    // #[error("{0}")]
    // RelatedRecordNotFound(String),
    // #[error("User (open_id: {0}) does not own this note (id: {1}).")]
    // NoOwnerPermission(OpenId, Uuid),
    // #[error("User (open_id: {0}) is not authorized to edit this note (id: {1}).")]
    // NoEditorPermission(OpenId, Uuid),
    // #[error("User (open_id: {0}) is not authorized to access this note (id: {1}).")]
    // UnauthorizedAccess(OpenId, Uuid),
    // #[error("User (open_id: {0}) does not have lock on this note (id: {1}).")]
    // UserHasNoLock(OpenId, Uuid),
    // #[error("User (open_id: {0}) already has lock on this note (id: {1}).")]
    // LockedByOther(OpenId, Uuid),
    // #[error("ActionGoal (id: {0}) is shared with others.")]
    // SharedWithOther(Uuid),
    // #[error("ActionGoal (id: {0}) is secret.")]
    // SecretActionGoal(Uuid),
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
