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
    m20240927_000001_create_ambitions_table::Ambition,
    m20240927_000002_create_objectives_table::Objective,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AmbitionsObjectives::Table)
                    .if_not_exists()
                    .col(uuid(AmbitionsObjectives::AmbitionId))
                    .col(uuid(AmbitionsObjectives::ObjectiveId))
                    .primary_key(
                        Index::create()
                            .name("pk-ambitions_objectives")
                            .col(AmbitionsObjectives::AmbitionId)
                            .col(AmbitionsObjectives::ObjectiveId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ambitions_objectives-ambition_id")
                            .from(AmbitionsObjectives::Table, AmbitionsObjectives::AmbitionId)
                            .to(Ambition::Table, Ambition::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ambitions_objectives-objective_id")
                            .from(AmbitionsObjectives::Table, AmbitionsObjectives::ObjectiveId)
                            .to(Objective::Table, Objective::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AmbitionsObjectives::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum AmbitionsObjectives {
    Table,
    AmbitionId,
    ObjectiveId,
}
