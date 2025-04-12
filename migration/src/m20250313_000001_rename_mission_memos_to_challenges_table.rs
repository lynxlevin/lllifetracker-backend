use sea_orm_migration::prelude::{
    async_trait,
    sea_orm::{self, DeriveIden},
    DbErr, DeriveMigrationName, MigrationTrait, SchemaManager, Table,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(MissionMemo::Table, Challenge::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(MissionMemosTags::Table, ChallengesTags::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ChallengesTags::Table)
                    .rename_column(MissionMemosTags::MissionMemoId, ChallengesTags::ChallengeId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ChallengesTags::Table)
                    .rename_column(ChallengesTags::ChallengeId, MissionMemosTags::MissionMemoId)
                    .to_owned(),
            )
            .await?;
        manager
            .rename_table(
                Table::rename()
                    .table(ChallengesTags::Table, MissionMemosTags::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(Challenge::Table, MissionMemo::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum MissionMemo {
    Table,
}

#[derive(DeriveIden)]
pub enum Challenge {
    Table,
}

#[derive(DeriveIden)]
pub enum MissionMemosTags {
    Table,
    MissionMemoId,
}

#[derive(DeriveIden)]
pub enum ChallengesTags {
    Table,
    ChallengeId,
}
