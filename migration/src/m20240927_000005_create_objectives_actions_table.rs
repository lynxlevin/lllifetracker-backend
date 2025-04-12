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
    m20240927_000002_create_objectives_table::Objective,
    m20240927_000003_create_actions_table::Action,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ObjectivesActions::Table)
                    .if_not_exists()
                    .col(uuid(ObjectivesActions::ObjectiveId))
                    .col(uuid(ObjectivesActions::ActionId))
                    .primary_key(
                        Index::create()
                            .name("pk-objectives_actions")
                            .col(ObjectivesActions::ObjectiveId)
                            .col(ObjectivesActions::ActionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-objectives_actions-objective_id")
                            .from(ObjectivesActions::Table, ObjectivesActions::ObjectiveId)
                            .to(Objective::Table, Objective::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-objectives_actions-action_id")
                            .from(ObjectivesActions::Table, ObjectivesActions::ActionId)
                            .to(Action::Table, Action::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ObjectivesActions::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ObjectivesActions {
    Table,
    ObjectiveId,
    ActionId,
}
