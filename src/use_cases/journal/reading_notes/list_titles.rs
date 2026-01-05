use db_adapters::{
    reading_note_adapter::{
        ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteOrder, ReadingNoteQuery,
    },
    Order::Asc,
};
use entities::user as user_entity;

use crate::UseCaseError;

pub async fn list_reading_note_titles<'a>(
    user: user_entity::Model,
    reading_note_adapter: ReadingNoteAdapter<'a>,
) -> Result<Vec<String>, UseCaseError> {
    reading_note_adapter
        .filter_eq_user(&user)
        // FIXME: This should better be ordered by updated_at desc, but could this be done in a performant way?
        .order_by_title(Asc)
        .get_all_only_titles()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
