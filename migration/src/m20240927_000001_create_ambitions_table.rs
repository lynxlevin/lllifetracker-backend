use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, Expr, ForeignKey, ForeignKeyAction, MigrationTrait,
        SchemaManager, Table,
    },
    schema::{string, string_null, timestamp_with_time_zone, uuid},
};

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
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ambitions-user_id")
                            .from(Ambition::Table, Ambition::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
