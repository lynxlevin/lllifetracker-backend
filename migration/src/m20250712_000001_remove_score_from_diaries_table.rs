use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, Expr, Index, MigrationTrait, SchemaManager, Table,
    },
    schema::tiny_unsigned_null,
};

const UNIQUE_INDEX_NAME: &str = "diaries_user_id_date_unique_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Diary::Table)
                    .drop_column(Diary::Score)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(Index::drop().name(UNIQUE_INDEX_NAME).to_owned())
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(UNIQUE_INDEX_NAME)
                    .table(Diary::Table)
                    .col(Diary::UserId)
                    .col(Diary::Date)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Diary::Table)
                    .add_column_if_not_exists(
                        tiny_unsigned_null(Diary::Score)
                            .check(Expr::col(Diary::Score).gte(1))
                            .check(Expr::col(Diary::Score).lte(5)),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Diary {
    Table,
    UserId,
    Date,
    Score,
}
