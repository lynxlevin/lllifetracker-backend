use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod diary_query;

pub use diary_query::*;

#[derive(Debug, Error)]
pub enum DiaryServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
}
impl From<TransactionError<DiaryServiceError>> for DiaryServiceError {
    fn from(value: TransactionError<DiaryServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

#[derive(Clone)]
pub struct DiaryService<'a> {
    pub db: &'a DbConn,
}

impl<'a> DiaryService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
