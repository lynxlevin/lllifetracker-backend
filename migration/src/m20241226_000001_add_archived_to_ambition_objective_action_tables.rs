use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, MigrationTrait, SchemaManager, Table,
    },
    schema::boolean,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Ambition::Table)
                    .add_column_if_not_exists(boolean(Ambition::Archived).default(false))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Objective::Table)
                    .add_column_if_not_exists(boolean(Objective::Archived).default(false))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Action::Table)
                    .add_column_if_not_exists(boolean(Action::Archived).default(false))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Action::Table)
                    .drop_column(Action::Archived)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Objective::Table)
                    .drop_column(Objective::Archived)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Ambition::Table)
                    .drop_column(Objective::Archived)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Ambition {
    Table,
    Archived,
}

#[derive(DeriveIden)]
pub enum Objective {
    Table,
    Archived,
}

#[derive(DeriveIden)]
pub enum Action {
    Table,
    Archived,
}
