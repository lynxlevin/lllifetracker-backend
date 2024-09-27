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
                    .table(Objective::Table)
                    .if_not_exists()
                    .col(uuid(Objective::Id).primary_key())
                    .col(uuid(Objective::UserId))
                    .col(string(Objective::Name))
                    .col(
                        timestamp_with_time_zone(Objective::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Objective::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("objectives_user_fkey")
                    .from(Objective::Table, Objective::UserId)
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
                    .name("objectives_user_fkey")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Objective::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Objective {
    Table,
    Id,
    UserId,
    Name,
    CreatedAt,
    UpdatedAt,
}
