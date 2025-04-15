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
                    .table(Objective::Table, DesiredState::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(AmbitionsObjectives::Table, AmbitionsDesiredStates::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(AmbitionsDesiredStates::Table)
                    .rename_column(
                        AmbitionsObjectives::ObjectiveId,
                        AmbitionsDesiredStates::DesiredStateId,
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(ObjectivesActions::Table, DesiredStatesActions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(DesiredStatesActions::Table)
                    .rename_column(
                        ObjectivesActions::ObjectiveId,
                        DesiredStatesActions::DesiredStateId,
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .rename_column(Tag::ObjectiveId, Tag::DesiredStateId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .rename_column(Tag::DesiredStateId, Tag::ObjectiveId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DesiredStatesActions::Table)
                    .rename_column(
                        DesiredStatesActions::DesiredStateId,
                        ObjectivesActions::ObjectiveId,
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .rename_table(
                Table::rename()
                    .table(DesiredStatesActions::Table, ObjectivesActions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AmbitionsDesiredStates::Table)
                    .rename_column(
                        AmbitionsDesiredStates::DesiredStateId,
                        AmbitionsObjectives::ObjectiveId,
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .rename_table(
                Table::rename()
                    .table(AmbitionsDesiredStates::Table, AmbitionsObjectives::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(DesiredState::Table, Objective::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Objective {
    Table,
}

#[derive(DeriveIden)]
pub enum DesiredState {
    Table,
}

#[derive(DeriveIden)]
pub enum AmbitionsObjectives {
    Table,
    ObjectiveId,
}

#[derive(DeriveIden)]
pub enum AmbitionsDesiredStates {
    Table,
    DesiredStateId,
}

#[derive(DeriveIden)]
pub enum ObjectivesActions {
    Table,
    ObjectiveId,
}

#[derive(DeriveIden)]
pub enum DesiredStatesActions {
    Table,
    DesiredStateId,
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    ObjectiveId,
    DesiredStateId,
}
