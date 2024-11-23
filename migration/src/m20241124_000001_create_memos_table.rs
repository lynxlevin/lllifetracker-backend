use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240722_000001_create_users_table::User;

const INDEX_NAME: &str = "memos_user_id_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Memo::Table)
                    .if_not_exists()
                    .col(uuid(Memo::Id).primary_key())
                    .col(uuid(Memo::UserId))
                    .col(string_len(Memo::Title, 64))
                    .col(text(Memo::Text))
                    .col(date(Memo::Date))
                    .col(
                        timestamp_with_time_zone(Memo::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Memo::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-memos-user_id")
                            .from(Memo::Table, Memo::UserId)
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
                    .table(Memo::Table)
                    .col(Memo::UserId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Memo::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Memo {
    Table,
    Id,
    UserId,
    Title,
    Text,
    Date,
    CreatedAt,
    UpdatedAt,
}
