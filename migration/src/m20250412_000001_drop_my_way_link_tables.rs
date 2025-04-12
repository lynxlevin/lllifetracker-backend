use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AmbitionsDesiredStates::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DesiredStatesActions::Table).to_owned())
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DesiredStatesActions::Table)
                    .if_not_exists()
                    .col(uuid(DesiredStatesActions::DesiredStateId))
                    .col(uuid(DesiredStatesActions::ActionId))
                    .primary_key(
                        Index::create()
                            .name("pk-objectives_actions")
                            .col(DesiredStatesActions::DesiredStateId)
                            .col(DesiredStatesActions::ActionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-objectives_actions-objective_id")
                            .from(DesiredStatesActions::Table, DesiredStatesActions::DesiredStateId)
                            .to(DesiredState::Table, DesiredState::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-objectives_actions-action_id")
                            .from(DesiredStatesActions::Table, DesiredStatesActions::ActionId)
                            .to(Action::Table, Action::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(AmbitionsDesiredStates::Table)
                    .if_not_exists()
                    .col(uuid(AmbitionsDesiredStates::AmbitionId))
                    .col(uuid(AmbitionsDesiredStates::DesiredStateId))
                    .primary_key(
                        Index::create()
                            .name("pk-ambitions_objectives")
                            .col(AmbitionsDesiredStates::AmbitionId)
                            .col(AmbitionsDesiredStates::DesiredStateId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ambitions_objectives-ambition_id")
                            .from(AmbitionsDesiredStates::Table, AmbitionsDesiredStates::AmbitionId)
                            .to(Ambition::Table, Ambition::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ambitions_objectives-objective_id")
                            .from(AmbitionsDesiredStates::Table, AmbitionsDesiredStates::DesiredStateId)
                            .to(DesiredState::Table, DesiredState::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Ambition {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum DesiredState {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum Action {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum AmbitionsDesiredStates {
    Table,
    AmbitionId,
    DesiredStateId,
}

#[derive(DeriveIden)]
pub enum DesiredStatesActions {
    Table,
    DesiredStateId,
    ActionId,
}
