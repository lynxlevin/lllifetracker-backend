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
                    .table(Record::Table)
                    .if_not_exists()
                    .col(uuid(Record::Id).primary_key())
                    .col(uuid(Record::UserId))
                    .col(
                        timestamp_with_time_zone(Record::StartedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(Record::EndedAt))
                    .col(
                        timestamp_with_time_zone(Record::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Record::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("records_user_fkey")
                    .from(Record::Table, Record::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                sea_query::Index::create()
                    .name("records_user_id_started_at_index")
                    .table(Record::Table)
                    .col(Record::UserId)
                    .col(Record::StartedAt)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                sea_query::Index::drop()
                    .name("users_id_email_is_active_index")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("records_user_fkey")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Record::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Record {
    Table,
    Id,
    UserId,
    StartedAt,
    EndedAt,
    CreatedAt,
    UpdatedAt,
}
