use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod thinking_note_query;

pub use thinking_note_query::*;

#[derive(Debug, Error)]
pub enum ThinkingNoteServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
}
impl From<TransactionError<ThinkingNoteServiceError>> for ThinkingNoteServiceError {
    fn from(value: TransactionError<ThinkingNoteServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

#[derive(Clone)]
pub struct ThinkingNoteService<'a> {
    pub db: &'a DbConn,
}

impl<'a> ThinkingNoteService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
