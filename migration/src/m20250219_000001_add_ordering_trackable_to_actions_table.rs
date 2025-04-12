use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, MigrationTrait, SchemaManager, Table,
    },
    schema::{boolean, integer_null},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Action::Table)
                    .add_column_if_not_exists(integer_null(Action::Ordering))
                    .add_column_if_not_exists(boolean(Action::Trackable).default(true))
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
                    .drop_column(Action::Ordering)
                    .drop_column(Action::Trackable)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Action {
    Table,
    Ordering,
    Trackable,
}
