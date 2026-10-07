use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

// mod action_track_mutation;
mod action_track_query;

// pub use action_track_mutation::*;
pub use action_track_query::*;

#[derive(Debug, Error)]
pub enum ActionTrackServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    // #[error("ActionTrack cannot be found.")]
    // ActionTrackNotFound(),
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
    // #[error("ActionTrack (id: {0}) is shared with others.")]
    // SharedWithOther(Uuid),
    // #[error("ActionTrack (id: {0}) is secret.")]
    // SecretActionTrack(Uuid),
}
impl From<TransactionError<ActionTrackServiceError>> for ActionTrackServiceError {
    fn from(value: TransactionError<ActionTrackServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

#[derive(Clone)]
pub struct ActionTrackService<'a> {
    pub db: &'a DbConn,
}

impl<'a> ActionTrackService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
