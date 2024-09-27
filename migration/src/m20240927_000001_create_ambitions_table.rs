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
                    .table(Ambition::Table)
                    .if_not_exists()
                    .col(uuid(Ambition::Id).primary_key())
                    .col(uuid(Ambition::UserId))
                    .col(string(Ambition::Name))
                    .col(string_null(Ambition::Description))
                    .col(
                        timestamp_with_time_zone(Ambition::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Ambition::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("ambitions_user_fkey")
                    .from(Ambition::Table, Ambition::UserId)
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
                    .name("ambitions_user_fkey")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Ambition::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Ambition {
    Table,
    Id,
    UserId,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}
