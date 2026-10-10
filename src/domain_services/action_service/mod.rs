use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod action_mutation;
// mod action_query;

pub use action_mutation::*;
// pub use action_query::*;

#[derive(Debug, Error)]
pub enum ActionServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("Action cannot be found.")]
    ActionNotFound(),
    // #[error("Action cannot be found.")]
    // ActionNotFound(),
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
    // #[error("Action (id: {0}) is shared with others.")]
    // SharedWithOther(Uuid),
    // #[error("Action (id: {0}) is secret.")]
    // SecretAction(Uuid),
}
impl From<TransactionError<ActionServiceError>> for ActionServiceError {
    fn from(value: TransactionError<ActionServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

#[derive(Clone)]
pub struct ActionService<'a> {
    pub db: &'a DbConn,
}

impl<'a> ActionService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
