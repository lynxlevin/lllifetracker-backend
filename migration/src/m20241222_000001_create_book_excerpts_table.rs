use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240722_000001_create_users_table::User;

const INDEX_NAME: &str = "book_excerpts_user_id_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BookExcerpt::Table)
                    .if_not_exists()
                    .col(uuid(BookExcerpt::Id).primary_key())
                    .col(uuid(BookExcerpt::UserId))
                    .col(string_len(BookExcerpt::Title, 64))
                    .col(small_integer(BookExcerpt::PageNumber))
                    .col(text(BookExcerpt::Text))
                    .col(date(BookExcerpt::Date))
                    .col(
                        timestamp_with_time_zone(BookExcerpt::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(BookExcerpt::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-book-excerpts-user_id")
                            .from(BookExcerpt::Table, BookExcerpt::UserId)
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
                    .table(BookExcerpt::Table)
                    .col(BookExcerpt::UserId)
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
            .drop_table(Table::drop().table(BookExcerpt::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum BookExcerpt {
    Table,
    Id,
    UserId,
    Title,
    PageNumber,
    Text,
    Date,
    CreatedAt,
    UpdatedAt,
}
