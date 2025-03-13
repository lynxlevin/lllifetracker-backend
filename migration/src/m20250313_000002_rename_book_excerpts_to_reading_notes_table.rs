use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(BookExcerpt::Table, ReadingNote::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(BookExcerptsTags::Table, ReadingNotesTags::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ReadingNotesTags::Table)
                    .rename_column(BookExcerptsTags::BookExcerptId, ReadingNotesTags::ReadingNoteId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ReadingNotesTags::Table)
                    .rename_column(ReadingNotesTags::ReadingNoteId, BookExcerptsTags::BookExcerptId)
                    .to_owned(),
            )
            .await?;
        manager
            .rename_table(
                Table::rename()
                    .table(ReadingNotesTags::Table, BookExcerptsTags::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(ReadingNote::Table, BookExcerpt::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum BookExcerpt {
    Table,
}

#[derive(DeriveIden)]
pub enum ReadingNote {
    Table,
}

#[derive(DeriveIden)]
pub enum BookExcerptsTags {
    Table,
    BookExcerptId,
}

#[derive(DeriveIden)]
pub enum ReadingNotesTags {
    Table,
    ReadingNoteId,
}