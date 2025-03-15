use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Ambition::Table)
                    .add_column_if_not_exists(integer_null(Ambition::Ordering))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(DesiredState::Table)
                    .add_column_if_not_exists(integer_null(DesiredState::Ordering))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DesiredState::Table)
                    .drop_column(DesiredState::Ordering)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Ambition::Table)
                    .drop_column(Ambition::Ordering)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Ambition {
    Table,
    Ordering,
}

#[derive(DeriveIden)]
pub enum DesiredState {
    Table,
    Ordering,
}
