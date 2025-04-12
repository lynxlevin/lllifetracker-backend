use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, ForeignKeyAction, Index, MigrationTrait,
        SchemaManager, Table,
    },
    schema::uuid,
};

use crate::{m20240927_000006_create_tags_table::Tag, m20241124_000001_create_memos_table::Memo};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MemosTags::Table)
                    .if_not_exists()
                    .col(uuid(MemosTags::MemoId))
                    .col(uuid(MemosTags::TagId))
                    .primary_key(
                        Index::create()
                            .name("pk-memos_tags")
                            .col(MemosTags::MemoId)
                            .col(MemosTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-memos_tags-memo_id")
                            .from(MemosTags::Table, MemosTags::MemoId)
                            .to(Memo::Table, Memo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-memos_tags-tag_id")
                            .from(MemosTags::Table, MemosTags::TagId)
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
            .drop_table(Table::drop().table(MemosTags::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum MemosTags {
    Table,
    MemoId,
    TagId,
}
