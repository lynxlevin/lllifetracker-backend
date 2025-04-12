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
    m20240927_000006_create_tags_table::Tag,
    m20241218_000001_create_mission_memos_table::MissionMemo,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MissionMemosTags::Table)
                    .if_not_exists()
                    .col(uuid(MissionMemosTags::MissionMemoId))
                    .col(uuid(MissionMemosTags::TagId))
                    .primary_key(
                        Index::create()
                            .name("pk-mission_memos_tags")
                            .col(MissionMemosTags::MissionMemoId)
                            .col(MissionMemosTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mission_memos_tags-memo_id")
                            .from(MissionMemosTags::Table, MissionMemosTags::MissionMemoId)
                            .to(MissionMemo::Table, MissionMemo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mission_memos_tags-tag_id")
                            .from(MissionMemosTags::Table, MissionMemosTags::TagId)
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
            .drop_table(Table::drop().table(MissionMemosTags::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum MissionMemosTags {
    Table,
    MissionMemoId,
    TagId,
}
