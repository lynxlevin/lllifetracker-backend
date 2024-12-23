use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20240927_000006_create_tags_table::Tag,
    m20241222_000001_create_book_excerpts_table::BookExcerpt,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BookExcerptsTags::Table)
                    .if_not_exists()
                    .col(uuid(BookExcerptsTags::BookExcerptId))
                    .col(uuid(BookExcerptsTags::TagId))
                    .primary_key(
                        Index::create()
                            .name("pk-book_excerpts_tags")
                            .col(BookExcerptsTags::BookExcerptId)
                            .col(BookExcerptsTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-book_excerpts_tags-memo_id")
                            .from(BookExcerptsTags::Table, BookExcerptsTags::BookExcerptId)
                            .to(BookExcerpt::Table, BookExcerpt::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-book_excerpts_tags-tag_id")
                            .from(BookExcerptsTags::Table, BookExcerptsTags::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BookExcerptsTags::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum BookExcerptsTags {
    Table,
    BookExcerptId,
    TagId,
}
