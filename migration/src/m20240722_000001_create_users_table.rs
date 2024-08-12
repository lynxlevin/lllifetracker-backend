use chrono::prelude::Utc;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(uuid(User::Id).primary_key())
                    .col(string_uniq(User::Email))
                    .col(string(User::Password))
                    .col(string(User::FirstName))
                    .col(string(User::LastName))
                    .col(boolean(User::IsActive).default(false))
                    .col(timestamp_with_time_zone(User::CreatedAt).default(Utc::now()))
                    .col(timestamp_with_time_zone(User::UpdatedAt).default(Utc::now()))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                sea_query::Index::create()
                    .name("users_id_email_is_active_index")
                    .table(User::Table)
                    .col(User::Id)
                    .col(User::Email)
                    .col(User::IsActive)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    Password,
    FirstName,
    LastName,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
