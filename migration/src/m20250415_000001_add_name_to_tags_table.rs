use sea_orm_migration::{
    prelude::{
        async_trait, sea_orm::{self, DeriveIden}, DbErr, DeriveMigrationName, MigrationTrait,
        SchemaManager, Table,
    },
    schema::string_null,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .add_column_if_not_exists(string_null(Tag::Name))
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
                    .drop_column(Tag::Name)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    Name,
}
