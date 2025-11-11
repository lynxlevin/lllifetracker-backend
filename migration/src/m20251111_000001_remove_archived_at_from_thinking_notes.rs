use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, Expr, MigrationTrait, SchemaManager, Table,
    },
    schema::timestamp_with_time_zone_null,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ThinkingNote::Table)
                    .drop_column(ThinkingNote::ArchivedAt)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ThinkingNote::Table)
                    .add_column(
                        timestamp_with_time_zone_null(ThinkingNote::ArchivedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ThinkingNote {
    Table,
    ArchivedAt,
}
