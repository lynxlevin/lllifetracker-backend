use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod reading_note_query;

pub use reading_note_query::*;

#[derive(Debug, Error)]
pub enum ReadingNoteServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
}
impl From<TransactionError<ReadingNoteServiceError>> for ReadingNoteServiceError {
    fn from(value: TransactionError<ReadingNoteServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

#[derive(Clone)]
pub struct ReadingNoteService<'a> {
    pub db: &'a DbConn,
}

impl<'a> ReadingNoteService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
