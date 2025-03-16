use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240722_000001_create_users_table::User;

const INDEX_NAME: &str = "diaries_user_id_index";
const UNIQUE_INDEX_NAME: &str = "diaries_user_id_date_unique_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Diary::Table)
                    .if_not_exists()
                    .col(uuid(Diary::Id).primary_key())
                    .col(uuid(Diary::UserId))
                    .col(text_null(Diary::PositiveText))
                    .col(text_null(Diary::NegativeText))
                    .col(date(Diary::Date))
                    .col(
                        tiny_unsigned(Diary::Score)
                            .check(Expr::col(Diary::Score).gte(1))
                            .check(Expr::col(Diary::Score).lte(10)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-diaries-user_id")
                            .from(Diary::Table, Diary::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(INDEX_NAME)
                    .table(Diary::Table)
                    .col(Diary::UserId)
                    .to_owned(),
            )
            .await?;
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
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(UNIQUE_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Diary::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Diary {
    Table,
    Id,
    UserId,
    PositiveText,
    NegativeText,
    Date,
    Score,
}
