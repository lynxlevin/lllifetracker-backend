use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240722_000001_create_users_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Action::Table)
                    .if_not_exists()
                    .col(uuid(Action::Id).primary_key())
                    .col(uuid(Action::UserId))
                    .col(string(Action::Name))
                    .col(
                        timestamp_with_time_zone(Action::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Action::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("actions_user_fkey")
                    .from(Action::Table, Action::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("actions_user_fkey")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Action::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Action {
    Table,
    Id,
    UserId,
    Name,
    CreatedAt,
    UpdatedAt,
}
