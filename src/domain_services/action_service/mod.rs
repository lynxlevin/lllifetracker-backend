use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod action_mutation;
mod action_query;

pub use action_mutation::*;
pub use action_query::*;

#[derive(Debug, Error)]
pub enum ActionServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("Action cannot be found.")]
    ActionNotFound(),
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
