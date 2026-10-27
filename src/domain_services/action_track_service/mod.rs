use common::db::Db;
use sea_orm::{DbConn, DbErr, RuntimeErr::SqlxError, TransactionError};
use thiserror::Error;

mod action_track_mutation;
mod action_track_query;

pub use action_track_mutation::*;
pub use action_track_query::*;

#[derive(Debug, Error)]
pub enum ActionTrackServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("ActionTrack cannot be found.")]
    ActionTrackNotFound(),
    #[error("Action cannot be found.")]
    ActionNotFound(),
    #[error("ActionTrack for same action and started_at already exists.")]
    DuplicateCreation(),
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

fn handle_unique_constraint(error: DbErr) -> ActionTrackServiceError {
    match &error {
        DbErr::Query(SqlxError(err)) => match err.as_database_error() {
            Some(db_err) => match db_err.constraint() {
                Some("action_tracks_user_id_action_id_started_at_unique_index") => {
                    ActionTrackServiceError::DuplicateCreation()
                }
                Some("idx-action_track-action_tracks_user_id_action_id_started_at_uni") => {
                    ActionTrackServiceError::DuplicateCreation()
                }
                _ => error.into(),
            },
            None => error.into(),
        },
        _ => error.into(),
    }
}
