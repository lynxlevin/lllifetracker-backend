use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, ForeignKeyAction, Index, MigrationTrait,
        SchemaManager, Table,
    },
    schema::uuid,
};

use crate::{
    m20240927_000006_create_tags_table::Tag, m20250315_000002_create_diaries_table::Diary,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DiariesTags::Table)
                    .if_not_exists()
                    .col(uuid(DiariesTags::DiaryId))
                    .col(uuid(DiariesTags::TagId))
                    .primary_key(
                        Index::create()
                            .name("pk-diaries_tags")
                            .col(DiariesTags::DiaryId)
                            .col(DiariesTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-diaries_tags-diary_id")
                            .from(DiariesTags::Table, DiariesTags::DiaryId)
                            .to(Diary::Table, Diary::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-diaries_tags-tag_id")
                            .from(DiariesTags::Table, DiariesTags::TagId)
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
            .drop_table(Table::drop().table(DiariesTags::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum DiariesTags {
    Table,
    DiaryId,
    TagId,
}
